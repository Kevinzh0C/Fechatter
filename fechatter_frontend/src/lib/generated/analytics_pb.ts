// Auto-generated protobuf types for analytics
// Generated on: 2025-06-25T15:47:11.981Z
// Source: /Users/zhangkaiqi/Rust/Fechatter/fechatter_frontend/src/protos/analytics.proto

import * as protobuf from 'protobufjs';

// Load the protobuf root
export const analyticsProtoRoot = protobuf.Root.fromJSON({
  "nested": {
    "fechatter": {
      "nested": {
        "v1": {
          "nested": {
            "AnalyticsEvent": {
              "oneofs": {
                "eventType": {
                  "oneof": [
                    "appStart",
                    "appExit",
                    "userLogin",
                    "userLogout",
                    "userRegister",
                    "chatCreated",
                    "messageSent",
                    "chatJoined",
                    "chatLeft",
                    "navigation",
                    "fileUploaded",
                    "fileDownloaded",
                    "searchPerformed",
                    "notificationReceived",
                    "errorOccurred",
                    "botResponse"
                  ]
                }
              },
              "fields": {
                "context": {
                  "type": "EventContext",
                  "id": 1
                },
                "appStart": {
                  "type": "AppStartEvent",
                  "id": 10
                },
                "appExit": {
                  "type": "AppExitEvent",
                  "id": 11
                },
                "userLogin": {
                  "type": "UserLoginEvent",
                  "id": 12
                },
                "userLogout": {
                  "type": "UserLogoutEvent",
                  "id": 13
                },
                "userRegister": {
                  "type": "UserRegisterEvent",
                  "id": 14
                },
                "chatCreated": {
                  "type": "ChatCreatedEvent",
                  "id": 15
                },
                "messageSent": {
                  "type": "MessageSentEvent",
                  "id": 16
                },
                "chatJoined": {
                  "type": "ChatJoinedEvent",
                  "id": 17
                },
                "chatLeft": {
                  "type": "ChatLeftEvent",
                  "id": 18
                },
                "navigation": {
                  "type": "NavigationEvent",
                  "id": 19
                },
                "fileUploaded": {
                  "type": "FileUploadedEvent",
                  "id": 20
                },
                "fileDownloaded": {
                  "type": "FileDownloadedEvent",
                  "id": 21
                },
                "searchPerformed": {
                  "type": "SearchPerformedEvent",
                  "id": 22
                },
                "notificationReceived": {
                  "type": "NotificationReceivedEvent",
                  "id": 23
                },
                "errorOccurred": {
                  "type": "ErrorOccurredEvent",
                  "id": 24
                },
                "botResponse": {
                  "type": "BotResponseEvent",
                  "id": 25
                }
              }
            },
            "EventContext": {
              "fields": {
                "clientId": {
                  "type": "string",
                  "id": 1
                },
                "sessionId": {
                  "type": "string",
                  "id": 2
                },
                "userId": {
                  "type": "string",
                  "id": 3
                },
                "appVersion": {
                  "type": "string",
                  "id": 4
                },
                "clientTs": {
                  "type": "int64",
                  "id": 5
                },
                "serverTs": {
                  "type": "int64",
                  "id": 6
                },
                "userAgent": {
                  "type": "string",
                  "id": 7
                },
                "ip": {
                  "type": "string",
                  "id": 8
                },
                "system": {
                  "type": "SystemInfo",
                  "id": 9
                },
                "geo": {
                  "type": "GeoLocation",
                  "id": 10
                }
              }
            },
            "SystemInfo": {
              "fields": {
                "os": {
                  "type": "string",
                  "id": 1
                },
                "arch": {
                  "type": "string",
                  "id": 2
                },
                "locale": {
                  "type": "string",
                  "id": 3
                },
                "timezone": {
                  "type": "string",
                  "id": 4
                },
                "browser": {
                  "type": "string",
                  "id": 5
                },
                "browserVersion": {
                  "type": "string",
                  "id": 6
                }
              }
            },
            "GeoLocation": {
              "fields": {
                "country": {
                  "type": "string",
                  "id": 1
                },
                "region": {
                  "type": "string",
                  "id": 2
                },
                "city": {
                  "type": "string",
                  "id": 3
                }
              }
            },
            "AppStartEvent": {
              "fields": {}
            },
            "AppExitEvent": {
              "fields": {
                "exitCode": {
                  "type": "int32",
                  "id": 1
                }
              }
            },
            "UserLoginEvent": {
              "fields": {
                "email": {
                  "type": "string",
                  "id": 1
                },
                "loginMethod": {
                  "type": "string",
                  "id": 2
                }
              }
            },
            "UserLogoutEvent": {
              "fields": {
                "email": {
                  "type": "string",
                  "id": 1
                }
              }
            },
            "UserRegisterEvent": {
              "fields": {
                "email": {
                  "type": "string",
                  "id": 1
                },
                "workspaceId": {
                  "type": "string",
                  "id": 2
                },
                "registrationMethod": {
                  "type": "string",
                  "id": 3
                }
              }
            },
            "ChatCreatedEvent": {
              "fields": {
                "workspaceId": {
                  "type": "string",
                  "id": 1
                },
                "chatType": {
                  "type": "string",
                  "id": 2
                },
                "initialMembersCount": {
                  "type": "int32",
                  "id": 3
                }
              }
            },
            "MessageSentEvent": {
              "fields": {
                "chatId": {
                  "type": "string",
                  "id": 1
                },
                "messageType": {
                  "type": "string",
                  "id": 2
                },
                "messageLength": {
                  "type": "int32",
                  "id": 3
                },
                "hasAttachments": {
                  "type": "bool",
                  "id": 4
                },
                "hasMentions": {
                  "type": "bool",
                  "id": 5
                },
                "hasReactions": {
                  "type": "bool",
                  "id": 6
                }
              }
            },
            "ChatJoinedEvent": {
              "fields": {
                "chatId": {
                  "type": "string",
                  "id": 1
                },
                "joinMethod": {
                  "type": "string",
                  "id": 2
                }
              }
            },
            "ChatLeftEvent": {
              "fields": {
                "chatId": {
                  "type": "string",
                  "id": 1
                },
                "leaveReason": {
                  "type": "string",
                  "id": 2
                }
              }
            },
            "NavigationEvent": {
              "fields": {
                "from": {
                  "type": "string",
                  "id": 1
                },
                "to": {
                  "type": "string",
                  "id": 2
                },
                "durationMs": {
                  "type": "int32",
                  "id": 3
                }
              }
            },
            "FileUploadedEvent": {
              "fields": {
                "fileType": {
                  "type": "string",
                  "id": 1
                },
                "fileSize": {
                  "type": "int64",
                  "id": 2
                },
                "uploadMethod": {
                  "type": "string",
                  "id": 3
                },
                "uploadDurationMs": {
                  "type": "int32",
                  "id": 4
                }
              }
            },
            "FileDownloadedEvent": {
              "fields": {
                "fileType": {
                  "type": "string",
                  "id": 1
                },
                "fileSize": {
                  "type": "int64",
                  "id": 2
                },
                "downloadDurationMs": {
                  "type": "int32",
                  "id": 3
                }
              }
            },
            "SearchPerformedEvent": {
              "fields": {
                "searchType": {
                  "type": "string",
                  "id": 1
                },
                "queryLength": {
                  "type": "int32",
                  "id": 2
                },
                "resultsCount": {
                  "type": "int32",
                  "id": 3
                },
                "searchDurationMs": {
                  "type": "int32",
                  "id": 4
                },
                "hasFilters": {
                  "type": "bool",
                  "id": 5
                }
              }
            },
            "NotificationReceivedEvent": {
              "fields": {
                "notificationType": {
                  "type": "string",
                  "id": 1
                },
                "source": {
                  "type": "string",
                  "id": 2
                },
                "wasClicked": {
                  "type": "bool",
                  "id": 3
                }
              }
            },
            "ErrorOccurredEvent": {
              "fields": {
                "errorType": {
                  "type": "string",
                  "id": 1
                },
                "errorCode": {
                  "type": "string",
                  "id": 2
                },
                "errorMessage": {
                  "type": "string",
                  "id": 3
                },
                "stackTrace": {
                  "type": "string",
                  "id": 4
                },
                "context": {
                  "type": "string",
                  "id": 5
                }
              }
            },
            "BotResponseEvent": {
              "fields": {
                "botId": {
                  "type": "string",
                  "id": 1
                },
                "chatId": {
                  "type": "string",
                  "id": 2
                },
                "responseType": {
                  "type": "string",
                  "id": 3
                },
                "responseTimeMs": {
                  "type": "uint64",
                  "id": 4
                },
                "tokensUsed": {
                  "type": "uint32",
                  "id": 5
                },
                "success": {
                  "type": "bool",
                  "id": 6
                },
                "errorMessage": {
                  "type": "string",
                  "id": 7
                }
              }
            },
            "BatchRecordEventsRequest": {
              "fields": {
                "events": {
                  "rule": "repeated",
                  "type": "AnalyticsEvent",
                  "id": 1
                }
              }
            }
          }
        }
      }
    }
  }
});

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
