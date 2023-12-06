# \DataHubDataPoliciesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_data_policy**](DataHubDataPoliciesApi.md#create_data_policy) | **POST** /api/v1/data-hub/data-validation/policies | Create a new data policy
[**delete_data_policy**](DataHubDataPoliciesApi.md#delete_data_policy) | **DELETE** /api/v1/data-hub/data-validation/policies/{policyId} | Delete a data policy
[**get_all_data_policies**](DataHubDataPoliciesApi.md#get_all_data_policies) | **GET** /api/v1/data-hub/data-validation/policies | Get all data policies
[**get_data_policy**](DataHubDataPoliciesApi.md#get_data_policy) | **GET** /api/v1/data-hub/data-validation/policies/{policyId} | Get a data policy
[**update_data_policy**](DataHubDataPoliciesApi.md#update_data_policy) | **PUT** /api/v1/data-hub/data-validation/policies/{policyId} | Update an existing data policy



## create_data_policy

> crate::models::DataPolicy create_data_policy(data_policy)
Create a new data policy

Create a data policy  This endpoint requires at least HiveMQ version 4.15.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**data_policy** | [**DataPolicy**](DataPolicy.md) | The data policy to create. | [required] |

### Return type

[**crate::models::DataPolicy**](DataPolicy.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_data_policy

> delete_data_policy(policy_id)
Delete a data policy

Deletes an existing data policy.    

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_id** | **String** | The identifier of the data policy to delete. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_data_policies

> crate::models::DataPolicyList get_all_data_policies(fields, policy_ids, schema_ids, topic, limit, cursor)
Get all data policies

Get all data policies.    This endpoint returns the content of the policies with the content-type `application/json`.    This endpoint requires at least HiveMQ version 4.15.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, createdAt, lastUpdatedAt, matching, validation, onSuccess, onFailure |  |
**policy_ids** | Option<**String**> | Comma-separated list of policy IDs used for filtering. Multiple filters can be applied together. |  |
**schema_ids** | Option<**String**> | Comma-separated list of schema IDs used for filtering. Multiple filters can be applied together. |  |
**topic** | Option<**String**> | MQTT topic string that the retrieved policies must match. Returned policies are sorted in the same way as they are applied to matching publishes. 'topic' filtering does not support pagination |  |
**limit** | Option<**i32**> | Specifies the page size for the returned results. The value must be between 10 and 500. The default page size is 50. The limit is ignored if the 'topic' query parameter is set. |  |
**cursor** | Option<**String**> | The cursor that has been returned by the previous result page. Do not pass this parameter if you want to fetch the first page. |  |

### Return type

[**crate::models::DataPolicyList**](DataPolicyList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_data_policy

> crate::models::DataPolicy get_data_policy(policy_id, fields)
Get a data policy

Get a specific data policy.    This endpoint returns the content of the policy with the content-type `application/json`.    This endpoint requires at least HiveMQ version 4.15.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_id** | **String** | The identifier of the policy. | [required] |
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, createdAt, lastUpdatedAt, matching, validation, onSuccess, onFailure |  |

### Return type

[**crate::models::DataPolicy**](DataPolicy.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_data_policy

> crate::models::DataPolicy update_data_policy(policy_id, data_policy)
Update an existing data policy

Update a data policy  The path parameter 'policyId' must match the 'id' of the policy in the request body.  The matching part of policies cannot be changed with an update.  This endpoint requires at least HiveMQ version 4.17.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_id** | **String** | The identifier of the policy. | [required] |
**data_policy** | [**DataPolicy**](DataPolicy.md) | The data policy that should be updated. | [required] |

### Return type

[**crate::models::DataPolicy**](DataPolicy.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

