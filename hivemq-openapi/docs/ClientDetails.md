# ClientDetails

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connected** | Option<**bool**> | If this client is connected | [optional]
**connected_at** | Option<**String**> | Time the client connection was established | [optional]
**connection** | Option<[**crate::models::ConnectionDetails**](ConnectionDetails.md)> |  | [optional]
**id** | Option<**String**> | The MQTT client identifier | [optional]
**message_queue_size** | Option<**i64**> | The current message queue size for this client | [optional]
**restrictions** | Option<[**crate::models::ClientRestrictions**](ClientRestrictions.md)> |  | [optional]
**session_expiry_interval** | Option<**i64**> | The session expiry interval | [optional]
**will_present** | Option<**bool**> | If a will is present for this client | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


