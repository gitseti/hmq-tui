# \MqttClientsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**disconnect_client**](MqttClientsApi.md#disconnect_client) | **DELETE** /api/v1/mqtt/clients/{clientId}/connection | Disconnect a client
[**get_all_mqtt_clients**](MqttClientsApi.md#get_all_mqtt_clients) | **GET** /api/v1/mqtt/clients | List all MQTT clients
[**get_mqtt_client_connection_state**](MqttClientsApi.md#get_mqtt_client_connection_state) | **GET** /api/v1/mqtt/clients/{clientId}/connection | Get a clients connection state
[**get_mqtt_client_details**](MqttClientsApi.md#get_mqtt_client_details) | **GET** /api/v1/mqtt/clients/{clientId} | Get detailed client information
[**get_subscriptions_for_mqtt_client**](MqttClientsApi.md#get_subscriptions_for_mqtt_client) | **GET** /api/v1/mqtt/clients/{clientId}/subscriptions | List all subscriptions for MQTT client
[**invalidate_client_session**](MqttClientsApi.md#invalidate_client_session) | **DELETE** /api/v1/mqtt/clients/{clientId} | Invalidate a client session



## disconnect_client

> disconnect_client(client_id, prevent_will_message)
Disconnect a client

Disconnects a specific client if its is currently connected.   If your client identifiers contain special characters, please make sure that the clientId is URL Encoded (a.k.a. percent-encoding, as in RFC 3986).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The MQTT client identifier. | [required] |
**prevent_will_message** | Option<**bool**> | Whether to prevent the will message. |  |[default to false]

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_mqtt_clients

> crate::models::ClientList get_all_mqtt_clients(limit, cursor)
List all MQTT clients

Lists all client sessions (online and offline) known to the whole HiveMQ cluster.  The result contains each client's client identifier. For more details about each client you can call the endpoints that have a clientId in their URL.  This endpoint uses pagination with a cursor. The results are not sorted in any way, no ordering of any kind is guaranteed.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Specifies the page size for the returned results. Has to be between 50 and 2500. Default page size is 500. |  |
**cursor** | Option<**String**> | The cursor that has been returned by the previous result page. Do not pass this parameter if you want to fetch the first page. |  |

### Return type

[**crate::models::ClientList**](ClientList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_mqtt_client_connection_state

> crate::models::ConnectionItem get_mqtt_client_connection_state(client_id)
Get a clients connection state

Returns the information if a specific client is currently connected.   If you are only interested in the connection status of a client prefer this endpoint over the the full client detail. If your client identifiers contain special characters, please make sure that the clientId is URL Encoded (a.k.a. percent-encoding, as in RFC 3986).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The MQTT client identifier. | [required] |

### Return type

[**crate::models::ConnectionItem**](ConnectionItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_mqtt_client_details

> crate::models::ClientItem get_mqtt_client_details(client_id)
Get detailed client information

Returns detailed information for a specific client with it is current state.   Including all session and connection information. If your client identifiers contain special characters, please make sure that the clientId is URL Encoded (a.k.a. percent-encoding, as in RFC 3986).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The MQTT client identifier. | [required] |

### Return type

[**crate::models::ClientItem**](ClientItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_subscriptions_for_mqtt_client

> crate::models::ClientSubscriptionList get_subscriptions_for_mqtt_client(client_id)
List all subscriptions for MQTT client

List all subscriptions for a specific client.  This endpoint does not support pagination with cursor at the moment, but it might be added in future versions. Please make sure to check if a cursor is returned and another page is available to have a future-proof implementation.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The MQTT client identifier. | [required] |

### Return type

[**crate::models::ClientSubscriptionList**](ClientSubscriptionList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invalidate_client_session

> invalidate_client_session(client_id, prevent_will_message)
Invalidate a client session

Invalidates the client session for a client with the given client identifier. If the client is currently connected, it will be disconnected as well.   If your client identifiers contain special characters, please make sure that the clientId is URL encoded (a.k.a. percent-encoding, as in RFC 3986).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The MQTT client identifier. | [required] |
**prevent_will_message** | Option<**bool**> | Whether to prevent the will message. |  |[default to false]

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

