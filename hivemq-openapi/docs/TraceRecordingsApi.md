# \TraceRecordingsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_trace_recording**](TraceRecordingsApi.md#create_trace_recording) | **POST** /api/v1/management/trace-recordings | Create a trace recording
[**delete_trace_recording**](TraceRecordingsApi.md#delete_trace_recording) | **DELETE** /api/v1/management/trace-recordings/{traceRecordingId} | Delete a trace recording
[**download_trace_recording_file**](TraceRecordingsApi.md#download_trace_recording_file) | **GET** /api/v1/management/files/trace-recordings/{traceRecordingId} | Download a trace recording
[**get_all_trace_recordings**](TraceRecordingsApi.md#get_all_trace_recordings) | **GET** /api/v1/management/trace-recordings | Get all trace recordings
[**stop_trace_recording**](TraceRecordingsApi.md#stop_trace_recording) | **PATCH** /api/v1/management/trace-recordings/{traceRecordingId} | Stop a trace recording.



## create_trace_recording

> crate::models::TraceRecordingItem create_trace_recording(trace_recording_item)
Create a trace recording

Creates a new trace recording.    To create a trace recording you must specify a name, start date, end date, a set of filters and the desired packets that should be traced.  At least one client or topic filter and at least one packet is required to create a trace recording.  The client and topic filters can be [regular expressions](https://www.hivemq.com/docs/hivemq/4.3/control-center/analytic.html#regular-expressions).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**trace_recording_item** | Option<[**TraceRecordingItem**](TraceRecordingItem.md)> | The trace recording to create |  |

### Return type

[**crate::models::TraceRecordingItem**](TraceRecordingItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_trace_recording

> delete_trace_recording(trace_recording_id)
Delete a trace recording

Deletes an existing trace recording.    

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**trace_recording_id** | **String** | The name of the trace recording to delete. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_trace_recording_file

> std::path::PathBuf download_trace_recording_file(trace_recording_id)
Download a trace recording

Download a specific trace recording.    This endpoint returns the content of the trace recording with the content-type `application/zip`.   Only trace recordings in the states `IN_PROGRESS`, `STOPPED` and `ABORTED` can be downloaded.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**trace_recording_id** | **String** | The id of the trace recording. | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/zip, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_trace_recordings

> crate::models::TraceRecordingList get_all_trace_recordings()
Get all trace recordings

Lists all known trace recordings.   Trace recordings can be in different states. These states are: - `SCHEDULED` if the start date for a trace recording is in the future - `STOPPED` if a trace recording has reached its end date or was stopped manually - `IN_PROGRESS` when the trace recording is currently ongoing - `ABORTED` if the trace recording was aborted by the server 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::TraceRecordingList**](TraceRecordingList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stop_trace_recording

> crate::models::TraceRecordingItem stop_trace_recording(trace_recording_id, trace_recording_item)
Stop a trace recording.

Stops an existing trace recording.  Only the state of the trace recording can be set to `STOPPED` with this endpoint, changes to other fields are ignored.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**trace_recording_id** | **String** | The name of the trace recording to patch/stop. | [required] |
**trace_recording_item** | Option<[**TraceRecordingItem**](TraceRecordingItem.md)> | The trace recording to change |  |

### Return type

[**crate::models::TraceRecordingItem**](TraceRecordingItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

