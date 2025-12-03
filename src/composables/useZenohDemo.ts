import { ref } from "vue";

// Response type enum for queryable replies
export enum ResponseType {
  Sample = 0,
  Error = 1,
  Ignore = 2,
}

// Dummy types
export type Priority = number;
export type CongestionControl = string;
export type Reliability = string;
export type Locality = string;
export type QueryTarget = string;
export type ConsolidationMode = string;
export type SampleKind = number;

export interface LogEntry {
  timestamp: Date;
  type: "info" | "success" | "error" | "data";
  message: string;
  data?: Record<string, any>;
}

export interface SessionState {
  displayId: string;
  serverUrl: string;
  session: any;
  createdAt: Date;
  isConnecting: boolean;
}

export interface SubscriberState {
  displayId: string;
  sessionId: string;
  keyExpr: string;
  subscriber: any;
  createdAt: Date;
  options: any;
}

export interface PublisherPutParametersState {
  publicationKind: SampleKind;
  payload: string;
  payloadEmpty: boolean;
  encoding: string;
  customEncoding: boolean;
  priority: Priority | undefined;
  congestionControl: CongestionControl | undefined;
  express: boolean | undefined;
  attachment: string;
  attachmentEmpty: boolean;
  putOptionsJSON: any;
  updatePutOptionsJSON: () => void;
}

export interface PublisherState {
  displayId: string;
  sessionId: string;
  keyExpr: string;
  publisher: any;
  createdAt: Date;
  options: any;
  putParameters: PublisherPutParametersState;
}

export interface ReplyParametersState {
  publicationKind: SampleKind;
  keyExpr: string;
  payload: string;
  payloadEmpty: boolean;
  encoding: string;
  customEncoding: boolean;
  priority: Priority | undefined;
  congestionControl: CongestionControl | undefined;
  express: boolean | undefined;
  useTimestamp: boolean | undefined;
  attachment: string;
  attachmentEmpty: boolean;
  replyOptionsJSON: any;
  updateReplyOptionsJSON: () => void;
}

export interface ReplyErrParametersState {
  payload: string;
  payloadEmpty: boolean;
  encoding: string;
  customEncoding: boolean;
  replyErrOptionsJSON: any;
  updateReplyErrOptionsJSON: () => void;
}

export interface QueryableResponseParametersState {
  replyType: ResponseType;
  reply: ReplyParametersState;
  replyErr: ReplyErrParametersState;
}

export interface QueryableState {
  displayId: string;
  sessionId: string;
  keyExpr: string;
  queryable: any;
  createdAt: Date;
  options: any;
  responseParameters: QueryableResponseParametersState;
}

export interface QuerierGetParametersState {
  congestionControl: CongestionControl | undefined;
  priority: Priority | undefined;
  express: boolean | undefined;
  allowedDestination: Locality | undefined;
  encoding: string;
  customEncoding: boolean;
  payload: string;
  payloadEmpty: boolean;
  attachment: string;
  attachmentEmpty: boolean;
  timeout: number | undefined;
  timeoutEmpty: boolean;
  target: QueryTarget | undefined;
  consolidation: ConsolidationMode | undefined;
  acceptReplies: string | undefined;
  getOptionsJSON: any;
  updateGetOptionsJSON: () => void;
}

export interface QuerierState {
  displayId: string;
  sessionId: string;
  keyExpr: string;
  querier: any;
  createdAt: Date;
  options: any;
  getParameters: QuerierGetParametersState;
}

export interface LivelinessTokenState {
  displayId: string;
  sessionId: string;
  keyExpr: string;
  token: any;
  createdAt: Date;
}

export interface LivelinessSubscriberState {
  displayId: string;
  sessionId: string;
  keyExpr: string;
  subscriber: any;
  createdAt: Date;
  options: any;
}

// Dummy composable that returns empty/placeholder values
export async function useZenohDemo() {
  // State refs
  const serverUrl = ref("ws://localhost:10000");
  const isConnecting = ref(false);
  const activeSessions = ref<SessionState[]>([]);
  const selectedSessionId = ref<string | null>(null);
  const logEntries = ref<LogEntry[]>([]);
  const activeSubscribers = ref<SubscriberState[]>([]);
  const activePublishers = ref<PublisherState[]>([]);
  const activeQueryables = ref<QueryableState[]>([]);
  const activeQueriers = ref<QuerierState[]>([]);
  const activeLivelinessTokens = ref<LivelinessTokenState[]>([]);
  const activeLivelinessSubscribers = ref<LivelinessSubscriberState[]>([]);

  // Parameters
  const putParameters = {
    publicationKind: ref(0 as SampleKind),
    key: ref("demo/example/test"),
    value: ref(""),
    valueEmpty: ref(true),
    encoding: ref(""),
    customEncoding: ref(false),
    priority: ref(undefined as Priority | undefined),
    congestionControl: ref(undefined as CongestionControl | undefined),
    express: ref(undefined as boolean | undefined),
    reliability: ref(undefined as Reliability | undefined),
    allowedDestination: ref(undefined as Locality | undefined),
    attachment: ref(""),
    attachmentEmpty: ref(true),
  };

  const subscriberParameters = {
    key: ref("demo/example/**"),
    allowedOrigin: ref(undefined as Locality | undefined),
  };

  const publisherParameters = {
    key: ref("demo/example/publisher"),
    encoding: ref(""),
    customEncoding: ref(false),
    priority: ref(undefined as Priority | undefined),
    congestionControl: ref(undefined as CongestionControl | undefined),
    express: ref(undefined as boolean | undefined),
    reliability: ref(undefined as Reliability | undefined),
    allowedDestination: ref(undefined as Locality | undefined),
  };

  const queryableParameters = {
    key: ref("demo/example/computation/**"),
    complete: ref(undefined as boolean | undefined),
    allowedOrigin: ref(undefined as Locality | undefined),
  };

  const querierParameters = {
    key: ref("demo/example/*"),
    priority: ref(undefined as Priority | undefined),
    congestionControl: ref(undefined as CongestionControl | undefined),
    express: ref(undefined as boolean | undefined),
    allowedDestination: ref(undefined as Locality | undefined),
    target: ref(undefined as QueryTarget | undefined),
    consolidation: ref(undefined as ConsolidationMode | undefined),
    timeout: ref(undefined as number | undefined),
    timeoutEmpty: ref(true),
    acceptReplies: ref(undefined as string | undefined),
  };

  const livelinessTokenParameters = {
    key: ref("demo/example/token0"),
  };

  const livelinessSubscriberParameters = {
    key: ref("demo/example/**"),
    history: ref(undefined as boolean | undefined),
  };

  const livelinessGetParameters = {
    key: ref("demo/example/**"),
    timeout: ref(undefined as number | undefined),
    timeoutEmpty: ref(true),
  };

  const getParameters = {
    key: ref("demo/example/*"),
    encoding: ref(""),
    customEncoding: ref(false),
    priority: ref(undefined as Priority | undefined),
    congestionControl: ref(undefined as CongestionControl | undefined),
    allowedDestination: ref(undefined as Locality | undefined),
    express: ref(undefined as boolean | undefined),
    payload: ref(""),
    payloadEmpty: ref(true),
    attachment: ref(""),
    attachmentEmpty: ref(true),
    timeout: ref(undefined as number | undefined),
    timeoutEmpty: ref(true),
    target: ref(undefined as QueryTarget | undefined),
    consolidation: ref(undefined as ConsolidationMode | undefined),
    acceptReplies: ref(undefined as string | undefined),
  };

  // Options
  const sampleKindOptions = [
    { label: "Put", value: 0 },
    { label: "Delete", value: 1 },
  ];

  const responseTypeOptions = [
    { label: "Sample", value: ResponseType.Sample },
    { label: "Error", value: ResponseType.Error },
    { label: "Ignore", value: ResponseType.Ignore },
  ];

  const priorityOptions = [
    { label: "Real Time", value: 1 },
    { label: "Interactive High", value: 2 },
    { label: "Interactive Low", value: 3 },
    { label: "Data High", value: 4 },
    { label: "Data", value: 5 },
    { label: "Data Low", value: 6 },
    { label: "Background", value: 7 },
  ];

  const congestionControlOptions = [
    { label: "Drop", value: "drop" },
    { label: "Block", value: "block" },
  ];

  const reliabilityOptions = [
    { label: "Best Effort", value: "best_effort" },
    { label: "Reliable", value: "reliable" },
  ];

  const localityOptions = [
    { label: "Any", value: "any" },
    { label: "Remote", value: "remote" },
    { label: "Session Local", value: "session_local" },
  ];

  const encodingOptions = [
    { label: "text/plain", value: "text/plain" },
    { label: "application/json", value: "application/json" },
    { label: "application/octet-stream", value: "application/octet-stream" },
  ];

  const targetOptions = [
    { label: "Best Matching", value: "best_matching" },
    { label: "All", value: "all" },
    { label: "All Complete", value: "all_complete" },
  ];

  const consolidationOptions = [
    { label: "Auto", value: "auto" },
    { label: "None", value: "none" },
    { label: "Monotonic", value: "monotonic" },
    { label: "Latest", value: "latest" },
  ];

  const acceptRepliesOptions = [
    { label: "Replies", value: "replies" },
    { label: "Replies and Errors", value: "replies_and_errors" },
  ];

  const SampleKind = {
    PUT: 0,
    DELETE: 1,
  };

  // Dummy operations
  const connect = async () => {
    console.log("Connect (dummy)");
  };

  const disconnect = async (sessionId: string) => {
    console.log("Disconnect (dummy)", sessionId);
  };

  const selectSession = (sessionId: string) => {
    console.log("Select session (dummy)", sessionId);
  };

  const performPut = async () => {
    console.log("Perform put (dummy)");
  };

  const performGet = async () => {
    console.log("Perform get (dummy)");
  };

  const getSessionInfo = async () => {
    console.log("Get session info (dummy)");
  };

  const subscribe = async () => {
    console.log("Subscribe (dummy)");
  };

  const unsubscribe = async (subscriberId: string) => {
    console.log("Unsubscribe (dummy)", subscriberId);
  };

  const declarePublisher = async () => {
    console.log("Declare publisher (dummy)");
  };

  const undeclarePublisher = async (publisherId: string) => {
    console.log("Undeclare publisher (dummy)", publisherId);
  };

  const publishData = async (publisherId: string) => {
    console.log("Publish data (dummy)", publisherId);
  };

  const declareQueryable = async () => {
    console.log("Declare queryable (dummy)");
  };

  const undeclareQueryable = async (queryableId: string) => {
    console.log("Undeclare queryable (dummy)", queryableId);
  };

  const declareQuerier = async () => {
    console.log("Declare querier (dummy)");
  };

  const undeclareQuerier = async (querierId: string) => {
    console.log("Undeclare querier (dummy)", querierId);
  };

  const performQuerierGet = async (querierId: string) => {
    console.log("Perform querier get (dummy)", querierId);
  };

  const declareLivelinessToken = async () => {
    console.log("Declare liveliness token (dummy)");
  };

  const undeclareLivelinessToken = async (tokenId: string) => {
    console.log("Undeclare liveliness token (dummy)", tokenId);
  };

  const declareLivelinessSubscriber = async () => {
    console.log("Declare liveliness subscriber (dummy)");
  };

  const undeclareLivelinessSubscriber = async (subscriberId: string) => {
    console.log("Undeclare liveliness subscriber (dummy)", subscriberId);
  };

  const performLivelinessGet = async () => {
    console.log("Perform liveliness get (dummy)");
  };

  const clearLog = () => {
    logEntries.value = [];
  };

  return {
    // State
    serverUrl,
    isConnecting,
    activeSessions,
    selectedSessionId,
    putParameters,
    logEntries,
    activeSubscribers,
    activePublishers,
    activeQueryables,
    activeQueriers,
    activeLivelinessTokens,
    activeLivelinessSubscribers,
    subscriberParameters,
    publisherParameters,
    queryableParameters,
    querierParameters,
    livelinessTokenParameters,
    livelinessSubscriberParameters,
    livelinessGetParameters,
    getParameters,

    // Option arrays
    sampleKindOptions,
    responseTypeOptions,
    priorityOptions,
    congestionControlOptions,
    reliabilityOptions,
    localityOptions,
    encodingOptions,
    targetOptions,
    consolidationOptions,
    acceptRepliesOptions,

    // Enum values
    SampleKind,

    // Operations
    connect,
    getSessionInfo,
    disconnect,
    selectSession,
    performPut,
    performGet,
    subscribe,
    unsubscribe,
    declarePublisher,
    undeclarePublisher,
    publishData,
    declareQueryable,
    undeclareQueryable,
    declareQuerier,
    undeclareQuerier,
    performQuerierGet,
    declareLivelinessToken,
    undeclareLivelinessToken,
    declareLivelinessSubscriber,
    undeclareLivelinessSubscriber,
    performLivelinessGet,

    // App operations
    clearLog,
  };
}
