#!/usr/bin/env node

/**
 * Generate TypeScript definitions from protobuf files
 * Uses protobuf.js to generate static code
 */

import protobuf from 'protobufjs';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function generateProtobufTypes() {
  try {
    console.log('Generating protobuf TypeScript definitions...');

    // Load the proto file
    const protoPath = path.join(__dirname, '../src/protos/analytics.proto');
    const outputPath = path.join(__dirname, '../src/lib/generated/analytics_pb.ts');

    // Ensure output directory exists
    const outputDir = path.dirname(outputPath);
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    // Load and parse the proto file
    const root = await protobuf.load(protoPath);

    // Generate TypeScript definitions
    const code = `// Auto-generated protobuf types for analytics
// Generated on: ${new Date().toISOString()}
// Source: ${protoPath}

import * as protobuf from 'protobufjs';

// Load the protobuf root
export const analyticsProtoRoot = protobuf.Root.fromJSON(${JSON.stringify(root.toJSON(), null, 2)});

// Export message types
export const AnalyticsEvent = analyticsProtoRoot.lookupType('fechatter.v1.AnalyticsEvent');
export const EventContext = analyticsProtoRoot.lookupType('fechatter.v1.EventContext');
export const SystemInfo = analyticsProtoRoot.lookupType('fechatter.v1.SystemInfo');
export const GeoLocation = analyticsProtoRoot.lookupType('fechatter.v1.GeoLocation');
export const BatchRecordEventsRequest = analyticsProtoRoot.lookupType('fechatter.v1.BatchRecordEventsRequest');

// Event type messages
export const AppStartEvent = analyticsProtoRoot.lookupType('fechatter.v1.AppStartEvent');
export const AppExitEvent = analyticsProtoRoot.lookupType('fechatter.v1.AppExitEvent');
export const UserLoginEvent = analyticsProtoRoot.lookupType('fechatter.v1.UserLoginEvent');
export const UserLogoutEvent = analyticsProtoRoot.lookupType('fechatter.v1.UserLogoutEvent');
export const UserRegisterEvent = analyticsProtoRoot.lookupType('fechatter.v1.UserRegisterEvent');
export const ChatCreatedEvent = analyticsProtoRoot.lookupType('fechatter.v1.ChatCreatedEvent');
export const MessageSentEvent = analyticsProtoRoot.lookupType('fechatter.v1.MessageSentEvent');
export const ChatJoinedEvent = analyticsProtoRoot.lookupType('fechatter.v1.ChatJoinedEvent');
export const ChatLeftEvent = analyticsProtoRoot.lookupType('fechatter.v1.ChatLeftEvent');
export const NavigationEvent = analyticsProtoRoot.lookupType('fechatter.v1.NavigationEvent');
export const FileUploadedEvent = analyticsProtoRoot.lookupType('fechatter.v1.FileUploadedEvent');
export const FileDownloadedEvent = analyticsProtoRoot.lookupType('fechatter.v1.FileDownloadedEvent');
export const SearchPerformedEvent = analyticsProtoRoot.lookupType('fechatter.v1.SearchPerformedEvent');
export const NotificationReceivedEvent = analyticsProtoRoot.lookupType('fechatter.v1.NotificationReceivedEvent');
export const ErrorOccurredEvent = analyticsProtoRoot.lookupType('fechatter.v1.ErrorOccurredEvent');
export const BotResponseEvent = analyticsProtoRoot.lookupType('fechatter.v1.BotResponseEvent');

// TypeScript interfaces for better type safety
export interface IAnalyticsEvent {
  context?: IEventContext;
  eventType?: {
    appStart?: {};
    appExit?: { exitCode?: number };
    userLogin?: IUserLoginEvent;
    userLogout?: { email?: string };
    userRegister?: IUserRegisterEvent;
    chatCreated?: IChatCreatedEvent;
    messageSent?: IMessageSentEvent;
    chatJoined?: IChatJoinedEvent;
    chatLeft?: IChatLeftEvent;
    navigation?: INavigationEvent;
    fileUploaded?: IFileUploadedEvent;
    fileDownloaded?: IFileDownloadedEvent;
    searchPerformed?: ISearchPerformedEvent;
    notificationReceived?: INotificationReceivedEvent;
    errorOccurred?: IErrorOccurredEvent;
    botResponse?: IBotResponseEvent;
  };
}

export interface IEventContext {
  clientId?: string;
  sessionId?: string;
  userId?: string;
  appVersion?: string;
  clientTs?: number;
  serverTs?: number;
  userAgent?: string;
  ip?: string;
  system?: ISystemInfo;
  geo?: IGeoLocation;
}

export interface ISystemInfo {
  os?: string;
  arch?: string;
  locale?: string;
  timezone?: string;
  browser?: string;
  browserVersion?: string;
}

export interface IGeoLocation {
  country?: string;
  region?: string;
  city?: string;
}

export interface IUserLoginEvent {
  email?: string;
  loginMethod?: string;
}

export interface IUserRegisterEvent {
  email?: string;
  workspaceId?: string;
  registrationMethod?: string;
}

export interface IChatCreatedEvent {
  workspaceId?: string;
  chatType?: string;
  initialMembersCount?: number;
}

export interface IMessageSentEvent {
  chatId?: string;
  messageType?: string;
  messageLength?: number;
  hasAttachments?: boolean;
  hasMentions?: boolean;
  hasReactions?: boolean;
}

export interface IChatJoinedEvent {
  chatId?: string;
  joinMethod?: string;
}

export interface IChatLeftEvent {
  chatId?: string;
  leaveReason?: string;
}

export interface INavigationEvent {
  from?: string;
  to?: string;
  durationMs?: number;
}

export interface IFileUploadedEvent {
  fileType?: string;
  fileSize?: number;
  uploadMethod?: string;
  uploadDurationMs?: number;
}

export interface IFileDownloadedEvent {
  fileType?: string;
  fileSize?: number;
  downloadDurationMs?: number;
}

export interface ISearchPerformedEvent {
  searchType?: string;
  queryLength?: number;
  resultsCount?: number;
  searchDurationMs?: number;
  hasFilters?: boolean;
}

export interface INotificationReceivedEvent {
  notificationType?: string;
  source?: string;
  wasClicked?: boolean;
}

export interface IErrorOccurredEvent {
  errorType?: string;
  errorCode?: string;
  errorMessage?: string;
  stackTrace?: string;
  context?: string;
}

export interface IBotResponseEvent {
  botId?: string;
  chatId?: string;
  responseType?: string;
  responseTimeMs?: number;
  tokensUsed?: number;
  success?: boolean;
  errorMessage?: string;
}

export interface IBatchRecordEventsRequest {
  events?: IAnalyticsEvent[];
}

// Helper functions for encoding/decoding
export function encodeAnalyticsEvent(event: IAnalyticsEvent): Uint8Array {
  const message = AnalyticsEvent.create(event);
  return AnalyticsEvent.encode(message).finish();
}

export function decodeAnalyticsEvent(buffer: Uint8Array): IAnalyticsEvent {
  return AnalyticsEvent.decode(buffer) as any;
}

export function encodeBatchRequest(request: IBatchRecordEventsRequest): Uint8Array {
  const message = BatchRecordEventsRequest.create(request);
  return BatchRecordEventsRequest.encode(message).finish();
}

export function decodeBatchRequest(buffer: Uint8Array): IBatchRecordEventsRequest {
  return BatchRecordEventsRequest.decode(buffer) as any;
}
`;

    // Write the generated code
    fs.writeFileSync(outputPath, code);

    console.log('Successfully generated protobuf TypeScript definitions');
    console.log(`📁 Output: ${outputPath}`);

  } catch (error) {
    console.error('ERROR: Error generating protobuf types:', error);
    process.exit(1);
  }
}

// Run the generator
generateProtobufTypes(); 