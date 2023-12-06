# TraceRecording

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**client_id_filters** | Option<[**Vec<crate::models::TraceFilter>**](TraceFilter.md)> | Client ID filters to trace | [optional]
**end_at** | Option<**String**> | Time the trace recording is scheduled to stop at. Must be at a later time from the start time | [optional]
**events** | Option<**Vec<String>**> | MQTT events to trace | [optional]
**name** | Option<**String**> | Name of the trace recording. Must be unique, contain at least three characters and only combinations of numbers, letters, dashes and underscores are allowed | [optional]
**start_at** | Option<**String**> | Time the trace recording is scheduled to start at | [optional]
**state** | Option<**String**> | Current state of the recording. Only sent by the API, ignored if specified on POST | [optional]
**topic_filters** | Option<[**Vec<crate::models::TraceFilter>**](TraceFilter.md)> | Topic filters to trace | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


