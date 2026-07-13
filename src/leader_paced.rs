use crate::{
    adapters::{build_adapter, ProviderAdapter, RouteSelection, SendContext},
    artifacts::{
        summarize, ArtifactWriter, BenchManifest, BenchSample, BenchSampleContext, ManifestProvider,
    },
    collectors::{collect_rpcedge_observations, CollectRunOutput, RpcEdgeCollectConfig},
    config::{BenchConfig, ProviderSpec},
    leader_slots::{
        capture_leader_slots_snapshot, write_leader_slots_snapshot, LeaderSlotsCaptureConfig,
    },
    observations::{
        observation_summary_markdown, summarize_observations, EntryEvent, EntryPlacementEvent,
        MatchedObservationSummary, ObservationEvent, ObservationSourceKind,
    },
    provider_observers::add_default_provider_observation_accounts,
    slot_signal::{spawn_grpc_slot_signal, GrpcSlotSignalConfig, SlotSignal},
    tx::{
        build_transaction_with_blockhash, load_keypair, transaction_fee_lamports, BenchTx,
        TxBuildConfig,
    },
    url::redact_endpoint,
};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::signer::Signer;
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const LEADER_SLOTS_REFRESH_MARGIN: u64 = 16;
const DEFAULT_HARMONIC_MIN_CU_PRICE_MICROLAMPORTS: u64 = 300_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderPacedRouteStrategy {
    Static,
    ClientAware,
    AlwaysRace,
    SoftwareClientAware,
    PairedRoutePolicies,
    PairedProviders,
}

impl LeaderPacedRouteStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::ClientAware => "client_aware",
            Self::AlwaysRace => "always_race",
            Self::SoftwareClientAware => "software_client_aware",
            Self::PairedRoutePolicies => "paired_route_policies",
            Self::PairedProviders => "paired_providers",
        }
    }

    fn requires_leader_slots(self) -> bool {
        matches!(
            self,
            Self::ClientAware | Self::SoftwareClientAware | Self::PairedRoutePolicies
        )
    }

    fn is_paired(self) -> bool {
        matches!(self, Self::PairedRoutePolicies | Self::PairedProviders)
    }
}

#[derive(Debug, Clone)]
pub enum LeaderPacedTrigger {
    RpcPoll,
    GrpcSlot(GrpcSlotSignalConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderNetworkPathClass {
    Fra13Local,
    Doublezero,
    PublicInternet,
    Unresolved,
}

impl LeaderNetworkPathClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fra13Local => "fra13_local",
            Self::Doublezero => "doublezero",
            Self::PublicInternet => "public_internet",
            Self::Unresolved => "unresolved",
        }
    }
}

impl std::str::FromStr for LeaderNetworkPathClass {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.trim() {
            "fra13_local" | "fra13-local" => Ok(Self::Fra13Local),
            "doublezero" | "double_zero" | "double-zero" => Ok(Self::Doublezero),
            "public_internet" | "public-internet" => Ok(Self::PublicInternet),
            "unresolved" => Ok(Self::Unresolved),
            other => Err(format!(
                "unknown leader network path class `{other}`; expected `fra13_local`, `doublezero`, `public_internet`, or `unresolved`"
            )),
        }
    }
}

impl LeaderPacedTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RpcPoll => "rpc_poll",
            Self::GrpcSlot(_) => "grpc_slot",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LeaderPacedOptions {
    pub duration: Duration,
    pub txs_per_leader_run: usize,
    pub leader_run_concurrency: usize,
    pub lookbehind_slots: u64,
    pub lookahead_slots: u64,
    pub poll_interval: Duration,
    pub observe_extra: Duration,
    pub rpcedge: Option<RpcEdgeLeaderCollector>,
    pub leader_slots: Option<LeaderSlotsCaptureConfig>,
    pub route_strategy: LeaderPacedRouteStrategy,
    pub client_aware_harmonic_cu_price_microlamports: Option<u64>,
    pub trigger: LeaderPacedTrigger,
    pub extra_observation_accounts: Vec<String>,
    pub resend_attempts: usize,
    pub resend_interval: Duration,
    pub required_network_path_class: Option<LeaderNetworkPathClass>,
}

#[derive(Debug, Clone)]
pub struct RpcEdgeLeaderCollector {
    pub endpoint: String,
    pub x_token: Option<String>,
    pub min_sources: usize,
    pub collect_entries: bool,
}

#[derive(Debug)]
pub struct LeaderPacedRunOutput {
    pub test_id: String,
    pub run_dir: PathBuf,
    pub sent_transactions: usize,
    pub provider_samples: usize,
    pub collector: Option<CollectRunOutput>,
    pub matched_observation_summary: Option<MatchedObservationSummary>,
    pub leader_slots_snapshot_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderSendEvent {
    pub schema_version: u32,
    pub test_id: String,
    pub iteration: usize,
    pub signature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payer_pubkey: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_blockhash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_arm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_arm_index: Option<u16>,
    pub leader_identity: String,
    pub leader_run_start_slot: u64,
    pub leader_run_end_slot: u64,
    pub send_slot: u64,
    pub sent_at: DateTime<Utc>,
    pub trigger_source: String,
    pub slot_signal_status: Option<String>,
    pub slot_signal_observed_at: Option<DateTime<Utc>>,
    pub leader_client_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_software_client: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_software_client_id: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_network_path_class: Option<LeaderNetworkPathClass>,
    #[serde(default)]
    pub leader_network_path_verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_network_path_observed_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_network_path_rtt_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_stake: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader_data_center_key: Option<String>,
    pub route_policy: Option<String>,
    pub selected_routes: Vec<String>,
    pub compute_unit_limit: u32,
    pub compute_unit_price_microlamports: u64,
    pub base_fee_lamports: u64,
    pub priority_fee_lamports: u64,
    pub transaction_fee_lamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderRunDecisionEvent {
    pub schema_version: u32,
    pub test_id: String,
    pub leader_identity: String,
    pub leader_run_start_slot: u64,
    pub leader_run_end_slot: u64,
    pub observed_slot: u64,
    pub observed_at: DateTime<Utc>,
    pub eligible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_path_class: Option<LeaderNetworkPathClass>,
    #[serde(default)]
    pub network_path_verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_path_observed_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_path_rtt_us: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stake: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_center_key: Option<String>,
}

#[derive(Debug, Clone)]
struct LeaderRun {
    leader_identity: String,
    start_slot: u64,
    end_slot: u64,
}

#[derive(Debug, Clone)]
struct PolicyArm {
    comparison_group_id: Option<String>,
    policy_arm: Option<String>,
    policy_arm_index: Option<u16>,
    route_selection: Option<RouteSelection>,
    provider_index: Option<usize>,
}

#[derive(Debug)]
struct PendingLeaderTx {
    tx: BenchTx,
    iteration: usize,
    comparison_group_id: Option<String>,
    policy_arm: Option<String>,
    policy_arm_index: Option<u16>,
    route_selection: Option<RouteSelection>,
    provider_index: Option<usize>,
    payer_pubkey: String,
    recent_blockhash: String,
    leader_metadata: LeaderMetadata,
    compute_unit_limit: u32,
    compute_unit_price_microlamports: u64,
}

#[derive(Debug, Clone, Copy)]
struct ResendAttempt {
    index: usize,
    attempts: usize,
    interval: Duration,
}

pub async fn run_leader_paced(
    config: BenchConfig,
    options: LeaderPacedOptions,
) -> Result<LeaderPacedRunOutput> {
    if config.providers.is_empty() {
        bail!("at least one provider is required");
    }
    if options.txs_per_leader_run == 0 {
        bail!("txs_per_leader_run must be greater than zero");
    }
    if options.leader_run_concurrency == 0 {
        bail!("leader_run_concurrency must be greater than zero");
    }
    if options.resend_attempts == 0 {
        bail!("resend_attempts must be greater than zero");
    }
    if options.resend_attempts > u16::MAX as usize {
        bail!("resend_attempts must be <= {}", u16::MAX);
    }
    if options.route_strategy.requires_leader_slots() && options.leader_slots.is_none() {
        bail!(
            "{} route strategy requires --capture-leader-slots",
            options.route_strategy.as_str()
        );
    }
    if options.required_network_path_class.is_some() && options.leader_slots.is_none() {
        bail!("--leader-network-path-class requires --capture-leader-slots");
    }
    if options.route_strategy == LeaderPacedRouteStrategy::PairedProviders {
        validate_paired_providers(&config.providers)?;
    }
    validate_payer_config(options.route_strategy, &config)?;

    let test_id = config
        .test_id
        .clone()
        .unwrap_or_else(default_leader_paced_test_id);
    let max_spend_lamports = config
        .max_spend_lamports
        .context("max_spend_lamports is required for every transaction-producing run")?;
    if max_spend_lamports == 0 {
        bail!("max_spend_lamports must be greater than zero");
    }
    let payers = load_payers(options.route_strategy, &config)?;
    let payer_pubkeys = payers
        .iter()
        .map(|payer| payer.pubkey().to_string())
        .collect::<Vec<_>>();
    let memo_prefix = config
        .memo_prefix
        .clone()
        .unwrap_or_else(|| test_id.clone());
    let rpc = RpcClient::new_with_commitment(config.rpc_url.clone(), CommitmentConfig::processed());
    let timeout = Duration::from_millis(config.timeout_ms);
    let mut providers = config
        .providers
        .iter()
        .map(|spec| build_adapter(spec.name.clone(), spec.config.clone(), timeout))
        .collect::<Result<Vec<_>>>()?;
    for provider in providers.iter_mut() {
        provider.warmup().await?;
    }

    let mut writer = ArtifactWriter::create(&config.artifact_dir, &test_id)?;
    if matches!(options.trigger, LeaderPacedTrigger::GrpcSlot(_)) && options.leader_slots.is_none()
    {
        bail!("grpc slot trigger requires --capture-leader-slots");
    }

    let mut leader_lookup = LeaderMetadataLookup::default();
    let leader_slots_snapshot_path = if let (Some(leader_slots), LeaderPacedTrigger::RpcPoll) =
        (&options.leader_slots, &options.trigger)
    {
        let start_slot = rpc
            .get_slot()
            .context("get slot for getLeaderSlots snapshot")?;
        let artifact = refresh_leader_slots(leader_slots, start_slot, &writer.run_dir).await?;
        leader_lookup.extend_from_artifact(&artifact);
        write_leader_slots_snapshot(&writer.run_dir, &artifact)?;
        Some(writer.run_dir.join("leader-slots-snapshot.json"))
    } else {
        None
    };
    let leader_sends_path = writer.run_dir.join("leader-sends.ndjson");
    let mut leader_sends = BufWriter::new(
        File::create(&leader_sends_path)
            .with_context(|| format!("create {}", leader_sends_path.display()))?,
    );
    let leader_runs_path = writer.run_dir.join("leader-runs.ndjson");
    let mut leader_runs = BufWriter::new(
        File::create(&leader_runs_path)
            .with_context(|| format!("create {}", leader_runs_path.display()))?,
    );

    let collector_handle = options.rpcedge.clone().map(|rpcedge| {
        let output_dir = writer.run_dir.join("rpcedge-observations");
        let duration = options.duration + options.observe_extra;
        let mut account_include = payer_pubkeys.clone();
        add_default_provider_observation_accounts(&mut account_include);
        account_include.extend(options.extra_observation_accounts.iter().cloned());
        account_include.sort();
        account_include.dedup();
        let collector_config = RpcEdgeCollectConfig {
            test_id: test_id.clone(),
            endpoint: rpcedge.endpoint,
            x_token: rpcedge.x_token,
            output_dir,
            duration,
            account_include,
            min_sources: rpcedge.min_sources,
            collect_entries: rpcedge.collect_entries,
        };
        tokio::spawn(async move { collect_rpcedge_observations(collector_config).await })
    });

    let deadline = Instant::now() + options.duration;
    let mut evaluated_runs = HashSet::<(String, u64)>::new();
    let mut samples = Vec::<BenchSample>::new();
    let mut sends = Vec::<LeaderSendEvent>::new();
    let mut transaction_fee_spent = 0_u64;
    let mut iteration = 0_usize;

    let mut slot_rx = match options.trigger.clone() {
        LeaderPacedTrigger::RpcPoll => None,
        LeaderPacedTrigger::GrpcSlot(config) => Some(spawn_grpc_slot_signal(config)),
    };
    let trigger_source = options.trigger.as_str().to_string();

    while Instant::now() < deadline {
        let slot_signal = next_slot_signal(&rpc, slot_rx.as_mut(), deadline)
            .await
            .context("get next slot signal")?;
        let Some(slot_signal) = slot_signal else {
            break;
        };
        let send_slot = slot_signal.slot;
        if let Some(leader_slots) = options.leader_slots.as_ref() {
            let should_refresh = leader_lookup
                .max_slot()
                .map(|max_slot| send_slot.saturating_add(LEADER_SLOTS_REFRESH_MARGIN) >= max_slot)
                .unwrap_or(true)
                || leader_lookup.run_for_slot(send_slot).is_none();
            if should_refresh {
                let artifact =
                    refresh_leader_slots(leader_slots, send_slot, &writer.run_dir).await?;
                leader_lookup.extend_from_artifact(&artifact);
            }
        }
        let leader_run = match &options.trigger {
            LeaderPacedTrigger::RpcPoll if options.leader_slots.is_none() => current_leader_run(
                &rpc,
                send_slot,
                options.lookbehind_slots,
                options.lookahead_slots,
            )?,
            _ => leader_lookup.run_for_slot(send_slot).with_context(|| {
                format!("leader slot metadata missing for grpc-observed slot {send_slot}")
            })?,
        };
        let run_key = (leader_run.leader_identity.clone(), leader_run.start_slot);
        if evaluated_runs.insert(run_key) {
            let leader_metadata =
                leader_lookup.lookup(send_slot, leader_run.leader_identity.as_str());
            let eligible = options
                .required_network_path_class
                .map(|required| leader_matches_network_path(&leader_metadata, required))
                .unwrap_or(true);
            let decision = LeaderRunDecisionEvent {
                schema_version: 1,
                test_id: test_id.clone(),
                leader_identity: leader_run.leader_identity.clone(),
                leader_run_start_slot: leader_run.start_slot,
                leader_run_end_slot: leader_run.end_slot,
                observed_slot: send_slot,
                observed_at: slot_signal.observed_at,
                eligible,
                skip_reason: options.required_network_path_class.and_then(|required| {
                    (!eligible).then(|| leader_network_path_skip_reason(&leader_metadata, required))
                }),
                network_path_class: leader_metadata.network_path_class,
                network_path_verified: leader_metadata.network_path_verified,
                network_path_observed_at_unix_ms: leader_metadata.network_path_observed_at_unix_ms,
                network_path_rtt_us: leader_metadata.network_path_rtt_us,
                stake: leader_metadata.stake,
                region: leader_metadata.region.clone(),
                city: leader_metadata.city.clone(),
                data_center_key: leader_metadata.data_center_key.clone(),
            };
            serde_json::to_writer(&mut leader_runs, &decision)
                .context("write leader run decision")?;
            leader_runs
                .write_all(b"\n")
                .context("write leader run decision newline")?;
            leader_runs.flush().context("flush leader run decisions")?;

            if eligible {
                let blockhash = rpc.get_latest_blockhash().context("get latest blockhash")?;
                let mut pending = Vec::new();
                for group_index in 0..options.txs_per_leader_run {
                    let comparison_group_id =
                        comparison_group_id_for(&test_id, &leader_run, send_slot, group_index);
                    let mut arms =
                        if options.route_strategy == LeaderPacedRouteStrategy::PairedProviders {
                            paired_provider_arms(&config.providers, comparison_group_id)?
                        } else {
                            policy_arms_for_strategy(
                                options.route_strategy,
                                &leader_metadata,
                                comparison_group_id,
                            )
                        };
                    rotate_policy_arms(&mut arms, leader_run.start_slot, group_index);

                    for arm in arms {
                        let payer_index = if options.route_strategy
                            == LeaderPacedRouteStrategy::PairedProviders
                        {
                            paired_payer_index(
                                arm.policy_arm_index.context(
                                    "paired provider arm missing stable policy arm index",
                                )? as usize,
                                leader_run.start_slot,
                            )
                        } else {
                            0
                        };
                        let payer = payers
                            .get(payer_index)
                            .context("paired provider payer index out of bounds")?;
                        let compute_unit_price_microlamports = compute_unit_price_for_selection(
                            &config,
                            &options,
                            arm.route_selection.as_ref(),
                        );
                        let tx_config = TxBuildConfig {
                            rpc_url: config.rpc_url.clone(),
                            lamports: config.lamports,
                            compute_unit_limit: config.compute_unit_limit,
                            compute_unit_price_microlamports,
                        };
                        let next_transaction_fee_total = transaction_fee_spent
                            .saturating_add(transaction_fee_lamports(&tx_config));
                        if next_transaction_fee_total > max_spend_lamports {
                            bail!(
                                "spend cap exceeded before tx {iteration}: next_transaction_fee_total={} max={max}",
                                next_transaction_fee_total,
                                max = max_spend_lamports,
                            );
                        }

                        let tx = build_transaction_with_blockhash(
                            &tx_config, payer, iteration, blockhash,
                        )?;
                        transaction_fee_spent =
                            transaction_fee_spent.saturating_add(tx.transaction_fee_lamports);
                        pending.push(PendingLeaderTx {
                            tx,
                            iteration,
                            comparison_group_id: arm.comparison_group_id,
                            policy_arm: arm.policy_arm,
                            policy_arm_index: arm.policy_arm_index,
                            route_selection: arm.route_selection,
                            provider_index: arm.provider_index,
                            payer_pubkey: payer.pubkey().to_string(),
                            recent_blockhash: blockhash.to_string(),
                            leader_metadata: leader_metadata.clone(),
                            compute_unit_limit: tx_config.compute_unit_limit,
                            compute_unit_price_microlamports,
                        });
                        iteration += 1;
                    }
                }

                let chunk_size = if options.route_strategy.is_paired() {
                    pending.len().max(1)
                } else {
                    options.leader_run_concurrency
                };
                for chunk in pending.chunks(chunk_size) {
                    let send_batch_at = Utc::now();
                    for pending_tx in chunk {
                        let event = LeaderSendEvent {
                            schema_version: 4,
                            test_id: test_id.clone(),
                            iteration: pending_tx.iteration,
                            signature: pending_tx.tx.signature.to_string(),
                            payer_pubkey: Some(pending_tx.payer_pubkey.clone()),
                            recent_blockhash: Some(pending_tx.recent_blockhash.clone()),
                            comparison_group_id: pending_tx.comparison_group_id.clone(),
                            policy_arm: pending_tx.policy_arm.clone(),
                            policy_arm_index: pending_tx.policy_arm_index,
                            leader_identity: leader_run.leader_identity.clone(),
                            leader_run_start_slot: leader_run.start_slot,
                            leader_run_end_slot: leader_run.end_slot,
                            send_slot,
                            sent_at: send_batch_at,
                            trigger_source: trigger_source.clone(),
                            slot_signal_status: Some(slot_signal.status.clone()),
                            slot_signal_observed_at: Some(slot_signal.observed_at),
                            leader_client_family: pending_tx.leader_metadata.client_family.clone(),
                            leader_software_client: pending_tx
                                .leader_metadata
                                .software_client
                                .clone(),
                            leader_software_client_id: pending_tx
                                .leader_metadata
                                .software_client_id,
                            leader_network_path_class: pending_tx
                                .leader_metadata
                                .network_path_class,
                            leader_network_path_verified: pending_tx
                                .leader_metadata
                                .network_path_verified,
                            leader_network_path_observed_at_unix_ms: pending_tx
                                .leader_metadata
                                .network_path_observed_at_unix_ms,
                            leader_network_path_rtt_us: pending_tx
                                .leader_metadata
                                .network_path_rtt_us,
                            leader_stake: pending_tx.leader_metadata.stake,
                            leader_region: pending_tx.leader_metadata.region.clone(),
                            leader_city: pending_tx.leader_metadata.city.clone(),
                            leader_data_center_key: pending_tx
                                .leader_metadata
                                .data_center_key
                                .clone(),
                            route_policy: pending_tx
                                .route_selection
                                .as_ref()
                                .map(|selection| selection.policy.clone()),
                            selected_routes: pending_tx
                                .route_selection
                                .as_ref()
                                .map(|selection| selection.routes.clone())
                                .unwrap_or_default(),
                            compute_unit_limit: pending_tx.compute_unit_limit,
                            compute_unit_price_microlamports: pending_tx
                                .compute_unit_price_microlamports,
                            base_fee_lamports: pending_tx.tx.base_fee_lamports,
                            priority_fee_lamports: pending_tx.tx.priority_fee_lamports,
                            transaction_fee_lamports: pending_tx.tx.transaction_fee_lamports,
                        };
                        serde_json::to_writer(&mut leader_sends, &event)
                            .context("write leader send event")?;
                        leader_sends
                            .write_all(b"\n")
                            .context("write leader send newline")?;
                        sends.push(event);
                    }
                    leader_sends.flush().context("flush leader send events")?;

                    for resend_attempt_index in 0..options.resend_attempts {
                        let futures = chunk.iter().map(|pending_tx| {
                            send_one_transaction_to_providers(
                                providers.as_slice(),
                                test_id.as_str(),
                                pending_tx,
                                send_batch_at,
                                timeout,
                                ResendAttempt {
                                    index: resend_attempt_index,
                                    attempts: options.resend_attempts,
                                    interval: options.resend_interval,
                                },
                            )
                        });
                        for tx_samples in join_all(futures).await {
                            for sample in tx_samples {
                                writer.write_sample(&sample)?;
                                samples.push(sample);
                            }
                        }
                        if resend_attempt_index + 1 < options.resend_attempts
                            && !options.resend_interval.is_zero()
                        {
                            tokio::time::sleep(options.resend_interval).await;
                        }
                    }
                }
            }
        }
        if matches!(options.trigger, LeaderPacedTrigger::RpcPoll) {
            tokio::time::sleep(options.poll_interval).await;
        }
    }

    leader_sends.flush().context("flush leader sends")?;
    leader_runs.flush().context("flush leader run decisions")?;

    let collector = if let Some(handle) = collector_handle {
        Some(handle.await.context("join RPCEdge collector")??)
    } else {
        None
    };

    let matched_observation_summary = if let Some(collector) = collector.as_ref() {
        let min_sources = options
            .rpcedge
            .as_ref()
            .map(|rpcedge| rpcedge.min_sources)
            .unwrap_or(2);
        Some(write_matched_observation_summary(
            &writer.run_dir,
            &test_id,
            &samples,
            &collector.observations_path,
            collector.entries_path.as_deref(),
            min_sources,
        )?)
    } else {
        None
    };

    let manifest = BenchManifest {
        schema_version: 3,
        test_id: test_id.clone(),
        generated_at: Utc::now(),
        rpc_url_label: redact_endpoint(&config.rpc_url),
        keypair_pubkey: payer_pubkeys.first().cloned().unwrap_or_default(),
        keypair_pubkeys: payer_pubkeys,
        count: sends.len(),
        duration_seconds: Some(options.duration.as_secs()),
        rate_per_second: None,
        lamports: config.lamports,
        compute_unit_limit: config.compute_unit_limit,
        compute_unit_price_microlamports: config.compute_unit_price_microlamports,
        memo_prefix,
        max_spend_lamports: config.max_spend_lamports,
        route_strategy: Some(options.route_strategy.as_str().to_string()),
        leader_network_path_class: options
            .required_network_path_class
            .map(|class| class.as_str().to_string()),
        client_aware_harmonic_cu_price_microlamports: Some(harmonic_min_cu_price_microlamports(
            &options,
        )),
        resend_attempts: options.resend_attempts,
        resend_interval_ms: options.resend_interval.as_millis() as u64,
        providers: config
            .providers
            .iter()
            .map(|spec| ManifestProvider {
                name: spec.name.clone(),
                kind: spec.config.kind(),
                route_mode: spec
                    .config
                    .configured_route_mode()
                    .map(|mode| mode.as_wire().to_string()),
                routes: spec.config.configured_routes(),
            })
            .collect(),
    };
    writer.write_manifest(&manifest)?;
    let summary = summarize(&test_id, &samples);
    let run_dir = writer.finish(&summary)?;

    Ok(LeaderPacedRunOutput {
        test_id,
        run_dir,
        sent_transactions: sends.len(),
        provider_samples: samples.len(),
        collector,
        matched_observation_summary,
        leader_slots_snapshot_path,
    })
}

async fn next_slot_signal(
    rpc: &RpcClient,
    slot_rx: Option<&mut tokio::sync::mpsc::Receiver<SlotSignal>>,
    deadline: Instant,
) -> Result<Option<SlotSignal>> {
    if let Some(rx) = slot_rx {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(None);
        }
        return match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(signal)) => Ok(Some(signal)),
            Ok(None) | Err(_) => Ok(None),
        };
    }

    let slot = rpc.get_slot().context("get current slot")?;
    Ok(Some(SlotSignal {
        slot,
        status: "RPC_POLL".to_string(),
        observed_at: Utc::now(),
    }))
}

async fn refresh_leader_slots(
    config: &LeaderSlotsCaptureConfig,
    start_slot: u64,
    run_dir: &Path,
) -> Result<crate::leader_slots::LeaderSlotsSnapshotArtifact> {
    let artifact = capture_leader_slots_snapshot(config, start_slot).await?;
    let refresh_path = run_dir.join(format!("leader-slots-snapshot-{start_slot}.json"));
    fs::write(&refresh_path, serde_json::to_vec_pretty(&artifact)?)
        .with_context(|| format!("write {}", refresh_path.display()))?;
    Ok(artifact)
}

async fn send_one_transaction_to_providers(
    providers: &[Box<dyn ProviderAdapter>],
    test_id: &str,
    pending: &PendingLeaderTx,
    sent_at: DateTime<Utc>,
    timeout: Duration,
    resend: ResendAttempt,
) -> Vec<BenchSample> {
    let started = Instant::now();
    let ctx = SendContext {
        test_id: test_id.to_string(),
        iteration: pending.iteration,
        signature: pending.tx.signature.to_string(),
        tx_base64: pending.tx.base64.clone(),
        timeout,
        route_selection: pending.route_selection.clone(),
        leader_client_family: pending.leader_metadata.client_family.clone(),
    };
    let selected_providers = match pending.provider_index {
        Some(index) => providers.get(index).into_iter().collect::<Vec<_>>(),
        None => providers.iter().collect::<Vec<_>>(),
    };
    let futures = selected_providers
        .into_iter()
        .map(|provider| provider.send_transaction(&pending.tx.raw, &ctx));
    let acks = join_all(futures).await;
    let client_finished_at = Utc::now();
    let client_ack_latency_us = started.elapsed().as_micros();
    acks.into_iter()
        .map(|ack| {
            BenchSample::from_ack(
                BenchSampleContext {
                    test_id: test_id.to_string(),
                    iteration: pending.iteration,
                    signature: pending.tx.signature.to_string(),
                    comparison_group_id: pending.comparison_group_id.clone(),
                    policy_arm: pending.policy_arm.clone(),
                    policy_arm_index: pending.policy_arm_index,
                    client_started_at: sent_at,
                    client_finished_at,
                    client_ack_latency_us,
                    route_selection: pending.route_selection.clone(),
                    leader_client_family: pending.leader_metadata.client_family.clone(),
                    leader_software_client: pending.leader_metadata.software_client.clone(),
                    leader_software_client_id: pending.leader_metadata.software_client_id,
                    compute_unit_limit: pending.compute_unit_limit,
                    compute_unit_price_microlamports: pending.compute_unit_price_microlamports,
                    base_fee_lamports: pending.tx.base_fee_lamports,
                    priority_fee_lamports: pending.tx.priority_fee_lamports,
                    transaction_fee_lamports: pending.tx.transaction_fee_lamports,
                    resend_attempt_index: Some(resend.index as u16),
                    resend_attempts: Some(resend.attempts as u16),
                    resend_interval_ms: Some(resend.interval.as_millis() as u64),
                },
                ack,
            )
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
struct LeaderMetadata {
    client_family: Option<String>,
    software_client: Option<String>,
    software_client_id: Option<u16>,
    network_path_class: Option<LeaderNetworkPathClass>,
    network_path_verified: bool,
    network_path_observed_at_unix_ms: Option<u64>,
    network_path_rtt_us: Option<u64>,
    stake: Option<u64>,
    region: Option<String>,
    city: Option<String>,
    data_center_key: Option<String>,
}

#[derive(Debug, Clone)]
struct LeaderSlotInfo {
    identity: String,
    metadata: LeaderMetadata,
}

#[derive(Debug, Default)]
struct LeaderMetadataLookup {
    by_slot: BTreeMap<u64, LeaderSlotInfo>,
    by_identity: BTreeMap<String, LeaderMetadata>,
}

impl LeaderMetadataLookup {
    #[cfg(test)]
    fn from_artifact(artifact: &crate::leader_slots::LeaderSlotsSnapshotArtifact) -> Self {
        let mut lookup = Self::default();
        lookup.extend_from_artifact(artifact);
        lookup
    }

    fn extend_from_artifact(
        &mut self,
        artifact: &crate::leader_slots::LeaderSlotsSnapshotArtifact,
    ) {
        let data = artifact
            .response
            .get("result")
            .and_then(|result| result.get("data"))
            .and_then(Value::as_array)
            .or_else(|| artifact.response.get("data").and_then(Value::as_array));
        let Some(rows) = data else {
            return;
        };

        for row in rows {
            let Some(slot) = row.get("slot").and_then(Value::as_u64) else {
                continue;
            };
            let identity = row
                .get("identity")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let client_family = row
                .get("client")
                .and_then(|client| client.get("family"))
                .and_then(Value::as_str)
                .map(normalize_client_family);
            let software_client = row
                .get("client")
                .and_then(|client| client.get("software"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let software_client_id = row
                .get("client")
                .and_then(|client| client.get("softwareClientId"))
                .and_then(Value::as_u64)
                .and_then(|value| u16::try_from(value).ok());
            let network_path = row.get("networkPath");
            let network_path_class = network_path
                .and_then(|path| path.get("class"))
                .and_then(Value::as_str)
                .and_then(|value| value.parse().ok());
            let network_path_verified = network_path
                .and_then(|path| path.get("verified"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let network_path_observed_at_unix_ms = network_path
                .and_then(|path| path.get("observedAtUnixMs"))
                .and_then(Value::as_u64);
            let network_path_rtt_us = network_path
                .and_then(|path| path.get("rttUs"))
                .and_then(Value::as_u64);
            let stake = row.get("stake").and_then(Value::as_u64);
            let region = row
                .get("location")
                .and_then(|location| location.get("region"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let city = row
                .get("location")
                .and_then(|location| location.get("city"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let data_center_key = row
                .get("location")
                .and_then(|location| location.get("dataCenterKey"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let metadata = LeaderMetadata {
                client_family,
                software_client,
                software_client_id,
                network_path_class,
                network_path_verified,
                network_path_observed_at_unix_ms,
                network_path_rtt_us,
                stake,
                region,
                city,
                data_center_key,
            };
            if let Some(identity) = identity {
                self.by_slot.insert(
                    slot,
                    LeaderSlotInfo {
                        identity: identity.clone(),
                        metadata: metadata.clone(),
                    },
                );
                self.by_identity.entry(identity).or_insert(metadata);
            }
        }
    }

    fn lookup(&self, slot: u64, identity: &str) -> LeaderMetadata {
        self.by_slot
            .get(&slot)
            .map(|info| &info.metadata)
            .or_else(|| self.by_identity.get(identity))
            .cloned()
            .unwrap_or_default()
    }

    fn run_for_slot(&self, slot: u64) -> Option<LeaderRun> {
        let info = self.by_slot.get(&slot)?;
        let mut start_slot = slot;
        while start_slot > 0 {
            let previous = start_slot - 1;
            if self
                .by_slot
                .get(&previous)
                .map(|candidate| candidate.identity.as_str())
                == Some(info.identity.as_str())
            {
                start_slot = previous;
            } else {
                break;
            }
        }

        let mut end_slot = slot;
        loop {
            let next = end_slot.saturating_add(1);
            if self
                .by_slot
                .get(&next)
                .map(|candidate| candidate.identity.as_str())
                == Some(info.identity.as_str())
            {
                end_slot = next;
            } else {
                break;
            }
        }

        Some(LeaderRun {
            leader_identity: info.identity.clone(),
            start_slot,
            end_slot,
        })
    }

    fn max_slot(&self) -> Option<u64> {
        self.by_slot.keys().next_back().copied()
    }
}

fn leader_matches_network_path(
    metadata: &LeaderMetadata,
    required: LeaderNetworkPathClass,
) -> bool {
    metadata.network_path_verified && metadata.network_path_class == Some(required)
}

fn leader_network_path_skip_reason(
    metadata: &LeaderMetadata,
    required: LeaderNetworkPathClass,
) -> String {
    match metadata.network_path_class {
        None => "network_path_missing".to_string(),
        Some(_) if !metadata.network_path_verified => "network_path_unverified".to_string(),
        Some(actual) => format!(
            "network_path_class_mismatch_required_{}_actual_{}",
            required.as_str(),
            actual.as_str()
        ),
    }
}

fn normalize_client_family(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.contains("jito") {
        "jito".to_string()
    } else if normalized.contains("harmonic") {
        "harmonic".to_string()
    } else if normalized.contains("bam") {
        "bam".to_string()
    } else if normalized.contains("firedancer") || normalized.contains("frankendancer") {
        "firedancer".to_string()
    } else if normalized.contains("agave") || normalized.contains("anza") {
        "agave".to_string()
    } else {
        normalized
    }
}

fn route_selection_for_strategy(
    strategy: LeaderPacedRouteStrategy,
    leader_metadata: &LeaderMetadata,
) -> Option<RouteSelection> {
    match strategy {
        LeaderPacedRouteStrategy::Static => None,
        LeaderPacedRouteStrategy::AlwaysRace => Some(RouteSelection::only(
            "always_race",
            ["tpu_quic", "jito_bundle", "harmonic_bundle"],
        )),
        LeaderPacedRouteStrategy::ClientAware => {
            let family = leader_metadata
                .client_family
                .as_deref()
                .unwrap_or("unknown");
            if family == "jito" {
                Some(RouteSelection::only(
                    "client_aware_jito",
                    ["tpu_quic", "jito_bundle"],
                ))
            } else if family == "harmonic" {
                Some(RouteSelection::only(
                    "client_aware_harmonic",
                    ["tpu_quic", "harmonic_bundle"],
                ))
            } else {
                Some(RouteSelection::only("client_aware_tpu_only", ["tpu_quic"]))
            }
        }
        LeaderPacedRouteStrategy::SoftwareClientAware => {
            let routes = software_client_aware_routes(leader_metadata);
            let policy = match routes.as_slice() {
                ["tpu_quic", "jito_bundle", "harmonic_bundle"] => {
                    "software_client_aware_jito_harmonic"
                }
                ["tpu_quic", "jito_bundle"] => "software_client_aware_jito",
                ["tpu_quic", "harmonic_bundle"] => "software_client_aware_harmonic",
                _ => "software_client_aware_tpu_only",
            };
            Some(RouteSelection::only(policy, routes))
        }
        LeaderPacedRouteStrategy::PairedRoutePolicies => None,
        LeaderPacedRouteStrategy::PairedProviders => None,
    }
}

fn policy_arms_for_strategy(
    strategy: LeaderPacedRouteStrategy,
    leader_metadata: &LeaderMetadata,
    comparison_group_id: Option<String>,
) -> Vec<PolicyArm> {
    if strategy == LeaderPacedRouteStrategy::PairedRoutePolicies {
        return vec![
            PolicyArm {
                comparison_group_id: comparison_group_id.clone(),
                policy_arm: Some("tpu_quic_only".to_string()),
                policy_arm_index: Some(0),
                route_selection: Some(RouteSelection::only("tpu_quic_only", ["tpu_quic"])),
                provider_index: None,
            },
            PolicyArm {
                comparison_group_id: comparison_group_id.clone(),
                policy_arm: Some("always_race".to_string()),
                policy_arm_index: Some(1),
                route_selection: route_selection_for_strategy(
                    LeaderPacedRouteStrategy::AlwaysRace,
                    leader_metadata,
                ),
                provider_index: None,
            },
            PolicyArm {
                comparison_group_id,
                policy_arm: Some("software_client_aware".to_string()),
                policy_arm_index: Some(2),
                route_selection: route_selection_for_strategy(
                    LeaderPacedRouteStrategy::SoftwareClientAware,
                    leader_metadata,
                ),
                provider_index: None,
            },
        ];
    }

    vec![PolicyArm {
        comparison_group_id: None,
        policy_arm: None,
        policy_arm_index: None,
        route_selection: route_selection_for_strategy(strategy, leader_metadata),
        provider_index: None,
    }]
}

fn validate_paired_providers(providers: &[ProviderSpec]) -> Result<()> {
    if providers.len() != 2 {
        bail!(
            "paired_providers requires exactly two providers, got {}",
            providers.len()
        );
    }
    if providers[0].name == providers[1].name {
        bail!("paired_providers requires unique provider names");
    }
    for provider in providers {
        if provider.name.trim().is_empty() {
            bail!("paired_providers requires non-empty provider names");
        }
        if !provider.config.supports_route_selection() {
            bail!(
                "paired_providers provider {} does not support per-transaction route selection",
                provider.name
            );
        }
    }
    Ok(())
}

fn paired_provider_arms(
    providers: &[ProviderSpec],
    comparison_group_id: Option<String>,
) -> Result<Vec<PolicyArm>> {
    validate_paired_providers(providers)?;
    providers
        .iter()
        .enumerate()
        .map(|(index, provider)| {
            Ok(PolicyArm {
                comparison_group_id: comparison_group_id.clone(),
                policy_arm: Some(provider.name.clone()),
                policy_arm_index: Some(index as u16),
                route_selection: Some(RouteSelection::only(
                    format!("paired_provider_{}", provider.name),
                    ["tpu_quic"],
                )),
                provider_index: Some(index),
            })
        })
        .collect()
}

fn validate_payer_config(strategy: LeaderPacedRouteStrategy, config: &BenchConfig) -> Result<()> {
    if strategy == LeaderPacedRouteStrategy::PairedProviders
        && config.paired_payer_keypair_paths.len() != 2
    {
        bail!(
            "paired_providers requires exactly two paired_payer_keypair_paths, got {}",
            config.paired_payer_keypair_paths.len()
        );
    }
    Ok(())
}

fn load_payers(
    strategy: LeaderPacedRouteStrategy,
    config: &BenchConfig,
) -> Result<Vec<solana_sdk::signature::Keypair>> {
    if strategy == LeaderPacedRouteStrategy::PairedProviders {
        let payers = config
            .paired_payer_keypair_paths
            .iter()
            .map(|path| load_keypair(path))
            .collect::<Result<Vec<_>>>()?;
        if payers[0].pubkey() == payers[1].pubkey() {
            bail!("paired_providers requires two distinct payer keypairs");
        }
        return Ok(payers);
    }

    let keypair_path = config
        .keypair_path
        .as_ref()
        .context("keypair_path is required for transaction generation")?;
    Ok(vec![load_keypair(keypair_path)?])
}

fn paired_payer_index(policy_arm_index: usize, leader_run_start_slot: u64) -> usize {
    (policy_arm_index + (leader_run_start_slot / 4) as usize) % 2
}

fn rotate_policy_arms(arms: &mut [PolicyArm], leader_run_start_slot: u64, group_index: usize) {
    if arms.len() <= 1 {
        return;
    }
    let rotate_by = (leader_run_start_slot as usize + group_index) % arms.len();
    arms.rotate_left(rotate_by);
}

fn comparison_group_id_for(
    test_id: &str,
    leader_run: &LeaderRun,
    send_slot: u64,
    group_index: usize,
) -> Option<String> {
    Some(format!(
        "{test_id}:{}:{}:{}:{group_index}",
        leader_run.leader_identity, leader_run.start_slot, send_slot
    ))
}

fn compute_unit_price_for_selection(
    config: &BenchConfig,
    options: &LeaderPacedOptions,
    route_selection: Option<&RouteSelection>,
) -> u64 {
    if let Some(selection) = route_selection {
        if selection
            .routes
            .iter()
            .any(|route| route == "harmonic_bundle")
        {
            return config
                .compute_unit_price_microlamports
                .max(harmonic_min_cu_price_microlamports(options));
        }
    }

    config.compute_unit_price_microlamports
}

fn harmonic_min_cu_price_microlamports(options: &LeaderPacedOptions) -> u64 {
    options
        .client_aware_harmonic_cu_price_microlamports
        .unwrap_or(DEFAULT_HARMONIC_MIN_CU_PRICE_MICROLAMPORTS)
        .max(DEFAULT_HARMONIC_MIN_CU_PRICE_MICROLAMPORTS)
}

fn software_client_aware_routes(leader_metadata: &LeaderMetadata) -> Vec<&'static str> {
    let mut routes = vec!["tpu_quic"];
    if is_rakurai_software_client(leader_metadata) {
        routes.push("jito_bundle");
        routes.push("harmonic_bundle");
        return routes;
    }
    if is_jito_software_client(leader_metadata) {
        routes.push("jito_bundle");
    }
    if is_harmonic_software_client(leader_metadata) {
        routes.push("harmonic_bundle");
    }
    routes
}

fn is_jito_software_client(leader_metadata: &LeaderMetadata) -> bool {
    matches!(leader_metadata.software_client_id, Some(1 | 6 | 12))
        || leader_metadata
            .software_client
            .as_deref()
            .map(|value| {
                let value = value.to_ascii_lowercase();
                value.contains("jito") || value.contains("bam")
            })
            .unwrap_or(false)
}

fn is_harmonic_software_client(leader_metadata: &LeaderMetadata) -> bool {
    matches!(leader_metadata.software_client_id, Some(9..=11))
        || leader_metadata
            .software_client
            .as_deref()
            .map(|value| value.to_ascii_lowercase().contains("harmonic"))
            .unwrap_or(false)
}

fn is_rakurai_software_client(leader_metadata: &LeaderMetadata) -> bool {
    matches!(leader_metadata.software_client_id, Some(8))
        || leader_metadata
            .software_client
            .as_deref()
            .map(|value| {
                let value = value.to_ascii_lowercase();
                value.contains("rakurai") || value.contains("raiku")
            })
            .unwrap_or(false)
        || leader_metadata
            .client_family
            .as_deref()
            .map(|value| {
                let value = value.to_ascii_lowercase();
                value.contains("rakurai") || value.contains("raiku")
            })
            .unwrap_or(false)
}

fn current_leader_run(
    rpc: &RpcClient,
    current_slot: u64,
    lookbehind_slots: u64,
    lookahead_slots: u64,
) -> Result<LeaderRun> {
    let start_slot = current_slot.saturating_sub(lookbehind_slots);
    let limit = lookbehind_slots
        .saturating_add(lookahead_slots)
        .saturating_add(1)
        .max(1);
    let leaders = rpc
        .get_slot_leaders(start_slot, limit)
        .with_context(|| format!("get slot leaders start={start_slot} limit={limit}"))?;
    let current_index = current_slot.saturating_sub(start_slot) as usize;
    let leader = leaders
        .get(current_index)
        .with_context(|| format!("current slot {current_slot} missing from leader response"))?;

    let mut run_start_index = current_index;
    while run_start_index > 0 && leaders[run_start_index - 1] == *leader {
        run_start_index -= 1;
    }
    let mut run_end_index = current_index;
    while run_end_index + 1 < leaders.len() && leaders[run_end_index + 1] == *leader {
        run_end_index += 1;
    }

    Ok(LeaderRun {
        leader_identity: leader.to_string(),
        start_slot: start_slot + run_start_index as u64,
        end_slot: start_slot + run_end_index as u64,
    })
}

fn write_matched_observation_summary(
    run_dir: &std::path::Path,
    test_id: &str,
    samples: &[BenchSample],
    observations_path: &std::path::Path,
    entries_path: Option<&std::path::Path>,
    min_sources: usize,
) -> Result<MatchedObservationSummary> {
    #[derive(Debug, Clone)]
    struct MatchSubject {
        submitted_at: DateTime<Utc>,
        role: &'static str,
        caller_signature: String,
    }

    let mut submitted_at = BTreeMap::<String, MatchSubject>::new();
    for sample in samples {
        submitted_at
            .entry(sample.signature.clone())
            .and_modify(|existing| {
                if sample.client_started_at < existing.submitted_at {
                    existing.submitted_at = sample.client_started_at;
                }
            })
            .or_insert_with(|| MatchSubject {
                submitted_at: sample.client_started_at,
                role: "caller",
                caller_signature: sample.signature.clone(),
            });

        for result in &sample.route_results {
            let Some(tip_signature) = result.route_tip_signature.as_ref() else {
                continue;
            };
            submitted_at
                .entry(tip_signature.clone())
                .and_modify(|existing| {
                    if sample.client_started_at < existing.submitted_at {
                        existing.submitted_at = sample.client_started_at;
                    }
                })
                .or_insert_with(|| MatchSubject {
                    submitted_at: sample.client_started_at,
                    role: "route_tip",
                    caller_signature: sample.signature.clone(),
                });
        }
    }

    let file = File::open(observations_path)
        .with_context(|| format!("open {}", observations_path.display()))?;
    let mut matched_events = Vec::new();
    for (line_no, line) in BufReader::new(file).lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "read observation {}:{}",
                observations_path.display(),
                line_no + 1
            )
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let mut event = serde_json::from_str::<ObservationEvent>(&line).with_context(|| {
            format!(
                "parse observation {}:{}",
                observations_path.display(),
                line_no + 1
            )
        })?;
        if let Some(subject) = submitted_at.get(&event.signature) {
            event.submitted_at = Some(subject.submitted_at);
            event.signature_role = Some(subject.role.to_string());
            if subject.role != "caller" {
                event.caller_signature = Some(subject.caller_signature.clone());
            }
            matched_events.push(event);
        }
    }

    let matched_path = run_dir.join("matched-observations.ndjson");
    let mut matched_writer = BufWriter::new(
        File::create(&matched_path)
            .with_context(|| format!("create {}", matched_path.display()))?,
    );
    for event in &matched_events {
        serde_json::to_writer(&mut matched_writer, event).context("write matched observation")?;
        matched_writer
            .write_all(b"\n")
            .context("write matched observation newline")?;
    }
    matched_writer
        .flush()
        .context("flush matched observations")?;

    if let Some(entries_path) = entries_path {
        write_entry_placements(run_dir, test_id, &matched_events, entries_path)?;
    }

    let summary = summarize_observations(test_id, &matched_events, min_sources);
    fs::write(
        run_dir.join("matched-observation-summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )
    .context("write matched observation summary json")?;
    fs::write(
        run_dir.join("matched-observation-summary.md"),
        observation_summary_markdown(&summary),
    )
    .context("write matched observation summary markdown")?;
    Ok(summary)
}

fn write_entry_placements(
    run_dir: &std::path::Path,
    test_id: &str,
    matched_events: &[ObservationEvent],
    entries_path: &std::path::Path,
) -> Result<Vec<EntryPlacementEvent>> {
    let entries = read_entry_events(entries_path)?;
    let mut entries_by_slot = BTreeMap::<u64, Vec<EntryEvent>>::new();
    for entry in entries {
        entries_by_slot.entry(entry.slot).or_default().push(entry);
    }
    for entries in entries_by_slot.values_mut() {
        entries.sort_by_key(|entry| (entry.starting_transaction_index, entry.entry_index));
    }

    let mut placements = Vec::new();
    for event in matched_events {
        let (Some(slot), Some(slot_index)) = (event.slot, event.slot_index) else {
            continue;
        };
        let Some(entry) = entries_by_slot
            .get(&slot)
            .and_then(|entries| entry_for_slot_index(entries, slot_index))
        else {
            continue;
        };
        placements.push(EntryPlacementEvent {
            schema_version: 1,
            test_id: test_id.to_string(),
            signature: event.signature.clone(),
            signature_role: event.signature_role.clone(),
            caller_signature: event.caller_signature.clone(),
            source_name: entry.source_name.clone(),
            source_kind: ObservationSourceKind::YellowstoneEntry,
            observed_at: entry.observed_at,
            slot,
            slot_index,
            entry_index: entry.entry_index,
            tx_index_in_entry: slot_index.saturating_sub(entry.starting_transaction_index),
            entry_num_hashes: entry.num_hashes,
            entry_hash: entry.entry_hash.clone(),
            entry_executed_transaction_count: entry.executed_transaction_count,
            entry_starting_transaction_index: entry.starting_transaction_index,
        });
    }

    let placements_path = run_dir.join("entry-placements.ndjson");
    let mut writer = BufWriter::new(
        File::create(&placements_path)
            .with_context(|| format!("create {}", placements_path.display()))?,
    );
    for placement in &placements {
        serde_json::to_writer(&mut writer, placement).context("write entry placement")?;
        writer
            .write_all(b"\n")
            .context("write entry placement newline")?;
    }
    writer.flush().context("flush entry placements")?;
    Ok(placements)
}

fn read_entry_events(path: &std::path::Path) -> Result<Vec<EntryEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut rows = Vec::new();
    for (line_no, line) in BufReader::new(file).lines().enumerate() {
        let line = line.with_context(|| format!("read {}:{}", path.display(), line_no + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        rows.push(
            serde_json::from_str::<EntryEvent>(&line)
                .with_context(|| format!("parse {}:{}", path.display(), line_no + 1))?,
        );
    }
    Ok(rows)
}

fn entry_for_slot_index(entries: &[EntryEvent], slot_index: u64) -> Option<&EntryEvent> {
    entries.iter().find(|entry| {
        entry.executed_transaction_count > 0
            && slot_index >= entry.starting_transaction_index
            && slot_index
                < entry
                    .starting_transaction_index
                    .saturating_add(entry.executed_transaction_count)
    })
}

fn default_leader_paced_test_id() -> String {
    format!(
        "leader-paced-{}",
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    )
    .replace(':', "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapters::{ProviderConfig, RouteMode},
        config::ProviderSpec,
    };
    use serde_json::json;
    use std::collections::HashMap;

    fn route_aware_provider(name: &str) -> ProviderSpec {
        ProviderSpec {
            name: name.to_string(),
            config: ProviderConfig::RpcedgeRouteAwareHttp {
                endpoint: format!("http://127.0.0.1/{name}"),
                api_key_env: None,
                route_mode: RouteMode::Only,
                routes: vec!["tpu_quic".to_string()],
                headers: HashMap::new(),
            },
        }
    }

    #[test]
    fn paired_provider_arms_target_one_tpu_only_provider_each() {
        let providers = vec![
            route_aware_provider("public_internet"),
            route_aware_provider("doublezero"),
        ];

        let arms = paired_provider_arms(&providers, Some("group-1".to_string()))
            .expect("paired provider arms");

        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0].policy_arm.as_deref(), Some("public_internet"));
        assert_eq!(arms[0].policy_arm_index, Some(0));
        assert_eq!(arms[0].provider_index, Some(0));
        assert_eq!(arms[1].policy_arm.as_deref(), Some("doublezero"));
        assert_eq!(arms[1].policy_arm_index, Some(1));
        assert_eq!(arms[1].provider_index, Some(1));
        assert!(arms
            .iter()
            .all(|arm| arm.comparison_group_id.as_deref() == Some("group-1")));
        assert!(arms.iter().all(|arm| {
            let selection = arm.route_selection.as_ref().expect("route selection");
            selection.mode == RouteMode::Only && selection.routes == vec!["tpu_quic"]
        }));
    }

    #[test]
    fn paired_provider_validation_fails_closed() {
        let one = vec![route_aware_provider("public_internet")];
        assert!(paired_provider_arms(&one, Some("group".to_string())).is_err());

        let duplicate = vec![route_aware_provider("same"), route_aware_provider("same")];
        assert!(paired_provider_arms(&duplicate, Some("group".to_string())).is_err());

        let blank = vec![
            route_aware_provider("public_internet"),
            route_aware_provider(" "),
        ];
        assert!(paired_provider_arms(&blank, Some("group".to_string())).is_err());

        let incapable = vec![
            route_aware_provider("public_internet"),
            ProviderSpec {
                name: "doublezero".to_string(),
                config: ProviderConfig::SolanaRpc {
                    endpoint: "http://127.0.0.1:8899".to_string(),
                    headers: HashMap::new(),
                },
            },
        ];
        assert!(paired_provider_arms(&incapable, Some("group".to_string())).is_err());

        let three = vec![
            route_aware_provider("a"),
            route_aware_provider("b"),
            route_aware_provider("c"),
        ];
        assert!(paired_provider_arms(&three, Some("group".to_string())).is_err());
    }

    #[test]
    fn paired_payer_assignment_rotates_by_leader_run() {
        assert_ne!(paired_payer_index(0, 100), paired_payer_index(1, 100));
        assert_ne!(paired_payer_index(0, 104), paired_payer_index(1, 104));
        assert_ne!(paired_payer_index(0, 100), paired_payer_index(0, 104));
        assert_ne!(paired_payer_index(1, 100), paired_payer_index(1, 104));
    }

    #[test]
    fn paired_payer_validation_requires_exactly_two_paths_only_for_paired_providers() {
        let mut config = BenchConfig::default();
        assert!(validate_payer_config(LeaderPacedRouteStrategy::PairedProviders, &config).is_err());

        config.paired_payer_keypair_paths = vec![PathBuf::from("payer-a.json")];
        assert!(validate_payer_config(LeaderPacedRouteStrategy::PairedProviders, &config).is_err());

        config
            .paired_payer_keypair_paths
            .push(PathBuf::from("payer-b.json"));
        assert!(validate_payer_config(LeaderPacedRouteStrategy::PairedProviders, &config).is_ok());

        config
            .paired_payer_keypair_paths
            .push(PathBuf::from("payer-c.json"));
        assert!(validate_payer_config(LeaderPacedRouteStrategy::PairedProviders, &config).is_err());

        assert!(validate_payer_config(LeaderPacedRouteStrategy::Static, &config).is_ok());
    }

    #[test]
    fn client_aware_routes_jito_leaders_to_tpu_and_jito_bundle() {
        let metadata = LeaderMetadata {
            client_family: Some("jito".to_string()),
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::ClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "client_aware_jito");
        assert_eq!(selection.routes, vec!["tpu_quic", "jito_bundle"]);
    }

    #[test]
    fn client_aware_routes_harmonic_leaders_to_tpu_and_harmonic() {
        let metadata = LeaderMetadata {
            client_family: Some("harmonic".to_string()),
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::ClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "client_aware_harmonic");
        assert_eq!(selection.routes, vec!["tpu_quic", "harmonic_bundle"]);
    }

    #[test]
    fn client_aware_defaults_unknown_leaders_to_tpu_only() {
        let metadata = LeaderMetadata {
            client_family: Some("agave".to_string()),
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::ClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "client_aware_tpu_only");
        assert_eq!(selection.routes, vec!["tpu_quic"]);
    }

    #[test]
    fn always_race_routes_to_all_block_engine_paths() {
        let selection = route_selection_for_strategy(
            LeaderPacedRouteStrategy::AlwaysRace,
            &LeaderMetadata::default(),
        )
        .expect("selection");
        assert_eq!(selection.policy, "always_race");
        assert_eq!(
            selection.routes,
            vec!["tpu_quic", "jito_bundle", "harmonic_bundle"]
        );
    }

    #[test]
    fn software_client_aware_routes_bam_to_jito() {
        let metadata = LeaderMetadata {
            client_family: Some("bam".to_string()),
            software_client: Some("AgaveBam".to_string()),
            software_client_id: Some(6),
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::SoftwareClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "software_client_aware_jito");
        assert_eq!(selection.routes, vec!["tpu_quic", "jito_bundle"]);
    }

    #[test]
    fn software_client_aware_routes_harmonic_to_harmonic() {
        let metadata = LeaderMetadata {
            client_family: Some("harmonic".to_string()),
            software_client: Some("HarmonicAgave".to_string()),
            software_client_id: Some(10),
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::SoftwareClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "software_client_aware_harmonic");
        assert_eq!(selection.routes, vec!["tpu_quic", "harmonic_bundle"]);
    }

    #[test]
    fn software_client_aware_routes_rakurai_to_full_block_engine_race() {
        let metadata = LeaderMetadata {
            client_family: Some("rakurai".to_string()),
            software_client: Some("Rakurai".to_string()),
            software_client_id: Some(8),
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::SoftwareClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "software_client_aware_jito_harmonic");
        assert_eq!(
            selection.routes,
            vec!["tpu_quic", "jito_bundle", "harmonic_bundle"]
        );
    }

    #[test]
    fn software_client_aware_routes_raiku_to_full_block_engine_race() {
        let metadata = LeaderMetadata {
            client_family: Some("raiku".to_string()),
            software_client: Some("Raiku".to_string()),
            software_client_id: None,
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::SoftwareClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "software_client_aware_jito_harmonic");
        assert_eq!(
            selection.routes,
            vec!["tpu_quic", "jito_bundle", "harmonic_bundle"]
        );
    }

    #[test]
    fn software_client_aware_routes_plain_agave_to_tpu_only() {
        let metadata = LeaderMetadata {
            client_family: Some("agave".to_string()),
            software_client: Some("Agave".to_string()),
            software_client_id: Some(3),
            ..LeaderMetadata::default()
        };
        let selection =
            route_selection_for_strategy(LeaderPacedRouteStrategy::SoftwareClientAware, &metadata)
                .expect("selection");
        assert_eq!(selection.policy, "software_client_aware_tpu_only");
        assert_eq!(selection.routes, vec!["tpu_quic"]);
    }

    #[test]
    fn paired_policy_arms_include_all_route_policies_for_bam() {
        let metadata = LeaderMetadata {
            client_family: Some("bam".to_string()),
            software_client: Some("AgaveBam".to_string()),
            software_client_id: Some(6),
            ..LeaderMetadata::default()
        };
        let arms = policy_arms_for_strategy(
            LeaderPacedRouteStrategy::PairedRoutePolicies,
            &metadata,
            Some("group-1".to_string()),
        );
        assert_eq!(arms.len(), 3);
        assert_eq!(
            arms.iter()
                .map(|arm| arm.policy_arm.as_deref().unwrap_or(""))
                .collect::<Vec<_>>(),
            vec!["tpu_quic_only", "always_race", "software_client_aware"]
        );
        assert_eq!(
            arms.iter()
                .map(|arm| arm.policy_arm_index)
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), Some(2)]
        );
        assert!(arms
            .iter()
            .all(|arm| arm.comparison_group_id.as_deref() == Some("group-1")));
        assert_eq!(
            arms[0].route_selection.as_ref().unwrap().routes,
            vec!["tpu_quic"]
        );
        assert_eq!(
            arms[1].route_selection.as_ref().unwrap().routes,
            vec!["tpu_quic", "jito_bundle", "harmonic_bundle"]
        );
        assert_eq!(
            arms[2].route_selection.as_ref().unwrap().routes,
            vec!["tpu_quic", "jito_bundle"]
        );
    }

    #[test]
    fn paired_policy_arms_keep_plain_agave_software_arm_tpu_only() {
        let metadata = LeaderMetadata {
            client_family: Some("agave".to_string()),
            software_client: Some("Agave".to_string()),
            software_client_id: Some(3),
            ..LeaderMetadata::default()
        };
        let arms = policy_arms_for_strategy(
            LeaderPacedRouteStrategy::PairedRoutePolicies,
            &metadata,
            Some("group-2".to_string()),
        );
        let software = arms
            .iter()
            .find(|arm| arm.policy_arm.as_deref() == Some("software_client_aware"))
            .expect("software arm");
        let selection = software.route_selection.as_ref().expect("selection");
        assert_eq!(selection.policy, "software_client_aware_tpu_only");
        assert_eq!(selection.routes, vec!["tpu_quic"]);
    }

    #[test]
    fn paired_policy_arms_route_rakurai_software_arm_to_full_block_engine_race() {
        let metadata = LeaderMetadata {
            client_family: Some("rakurai".to_string()),
            software_client: Some("Rakurai".to_string()),
            software_client_id: Some(8),
            ..LeaderMetadata::default()
        };
        let arms = policy_arms_for_strategy(
            LeaderPacedRouteStrategy::PairedRoutePolicies,
            &metadata,
            Some("group-rakurai".to_string()),
        );
        let software = arms
            .iter()
            .find(|arm| arm.policy_arm.as_deref() == Some("software_client_aware"))
            .expect("software arm");
        let selection = software.route_selection.as_ref().expect("selection");
        assert_eq!(selection.policy, "software_client_aware_jito_harmonic");
        assert_eq!(
            selection.routes,
            vec!["tpu_quic", "jito_bundle", "harmonic_bundle"]
        );
    }

    #[test]
    fn route_selection_applies_default_harmonic_min_cu_price() {
        let config = BenchConfig {
            compute_unit_price_microlamports: 0,
            ..BenchConfig::default()
        };
        let mut options = LeaderPacedOptions {
            duration: Duration::from_secs(1),
            txs_per_leader_run: 1,
            leader_run_concurrency: 1,
            lookbehind_slots: 0,
            lookahead_slots: 0,
            poll_interval: Duration::from_millis(1),
            observe_extra: Duration::from_secs(0),
            rpcedge: None,
            leader_slots: None,
            route_strategy: LeaderPacedRouteStrategy::PairedRoutePolicies,
            client_aware_harmonic_cu_price_microlamports: None,
            trigger: LeaderPacedTrigger::RpcPoll,
            extra_observation_accounts: Vec::new(),
            resend_attempts: 1,
            resend_interval: Duration::from_millis(0),
            required_network_path_class: None,
        };
        let selection = RouteSelection::only("always_race", ["tpu_quic", "harmonic_bundle"]);
        assert_eq!(
            compute_unit_price_for_selection(&config, &options, Some(&selection)),
            300_000
        );
        let selection = RouteSelection::only("tpu_quic_only", ["tpu_quic"]);
        assert_eq!(
            compute_unit_price_for_selection(&config, &options, Some(&selection)),
            0
        );

        options.client_aware_harmonic_cu_price_microlamports = Some(1);
        let selection = RouteSelection::only("harmonic", ["harmonic_bundle"]);
        assert_eq!(
            compute_unit_price_for_selection(&config, &options, Some(&selection)),
            300_000
        );

        let config = BenchConfig {
            compute_unit_price_microlamports: 500_000,
            ..BenchConfig::default()
        };
        let selection = RouteSelection::only("harmonic", ["harmonic_bundle"]);
        assert_eq!(
            compute_unit_price_for_selection(&config, &options, Some(&selection)),
            500_000
        );
    }

    #[test]
    fn leader_slots_artifact_lookup_normalizes_client_family() {
        let artifact = crate::leader_slots::LeaderSlotsSnapshotArtifact {
            schema_version: 1,
            fetched_at: Utc::now(),
            rpc_url_label: "https://rpc.rpcedge.com/".to_string(),
            start_slot: 10,
            limit: 2,
            response: json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "success": true,
                    "data": [
                        {
                            "slot": 10,
                            "identity": "leader-a",
                            "client": {
                                "family": "JitoLabs",
                                "software": "JitoLabs",
                                "softwareClientId": 1
                            }
                        }
                    ]
                }
            }),
        };
        let lookup = LeaderMetadataLookup::from_artifact(&artifact);
        assert_eq!(
            lookup.lookup(10, "leader-a").client_family.as_deref(),
            Some("jito")
        );
        assert_eq!(
            lookup.lookup(10, "leader-a").software_client.as_deref(),
            Some("JitoLabs")
        );
        assert_eq!(lookup.lookup(10, "leader-a").software_client_id, Some(1));
        assert_eq!(
            lookup.lookup(11, "leader-a").client_family.as_deref(),
            Some("jito")
        );
    }

    fn network_path_artifact(
        class: Option<&str>,
        verified: Option<bool>,
    ) -> crate::leader_slots::LeaderSlotsSnapshotArtifact {
        let network_path = match (class, verified) {
            (Some(class), Some(verified)) => json!({
                "class": class,
                "verified": verified,
                "observedAtUnixMs": 1_234,
                "rttUs": 850,
                "latencySource": "quic_handshake"
            }),
            _ => Value::Null,
        };
        crate::leader_slots::LeaderSlotsSnapshotArtifact {
            schema_version: 1,
            fetched_at: Utc::now(),
            rpc_url_label: "https://rpc.rpcedge.com/".to_string(),
            start_slot: 20,
            limit: 1,
            response: json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "success": true,
                    "data": [{
                        "slot": 20,
                        "identity": "leader-dz",
                        "stake": 99,
                        "location": {
                            "region": "us-east",
                            "city": "New York",
                            "dataCenterKey": "us-east-1"
                        },
                        "networkPath": network_path,
                        "doublezeroEdge": {
                            "publicationStatus": "missing",
                            "leaderPublisher": false,
                            "observedLeaderSlots": 0
                        }
                    }]
                }
            }),
        }
    }

    #[test]
    fn leader_network_path_accepts_only_verified_doublezero() {
        let artifact = network_path_artifact(Some("doublezero"), Some(true));
        let metadata = LeaderMetadataLookup::from_artifact(&artifact).lookup(20, "leader-dz");

        assert!(leader_matches_network_path(
            &metadata,
            LeaderNetworkPathClass::Doublezero
        ));
        assert_eq!(metadata.network_path_observed_at_unix_ms, Some(1_234));
        assert_eq!(metadata.network_path_rtt_us, Some(850));
        assert_eq!(metadata.stake, Some(99));
        assert_eq!(metadata.region.as_deref(), Some("us-east"));
        assert_eq!(metadata.city.as_deref(), Some("New York"));
        assert_eq!(metadata.data_center_key.as_deref(), Some("us-east-1"));
    }

    #[test]
    fn leader_network_path_rejects_missing_unverified_and_other_classes() {
        for (class, verified) in [
            (Some("doublezero"), Some(false)),
            (Some("public_internet"), Some(true)),
            (Some("fra13_local"), Some(true)),
            (Some("unresolved"), Some(false)),
            (None, None),
        ] {
            let artifact = network_path_artifact(class, verified);
            let metadata = LeaderMetadataLookup::from_artifact(&artifact).lookup(20, "leader-dz");
            assert!(
                !leader_matches_network_path(&metadata, LeaderNetworkPathClass::Doublezero),
                "unexpected eligible route class={class:?} verified={verified:?}"
            );
        }
    }
}
