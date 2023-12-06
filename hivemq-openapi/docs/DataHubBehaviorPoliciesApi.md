# \DataHubBehaviorPoliciesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_behavior_policy**](DataHubBehaviorPoliciesApi.md#create_behavior_policy) | **POST** /api/v1/data-hub/behavior-validation/policies | Create a new policy
[**delete_behavior_policy**](DataHubBehaviorPoliciesApi.md#delete_behavior_policy) | **DELETE** /api/v1/data-hub/behavior-validation/policies/{policyId} | Delete a behavior policy
[**get_all_behavior_policies**](DataHubBehaviorPoliciesApi.md#get_all_behavior_policies) | **GET** /api/v1/data-hub/behavior-validation/policies | Get all policies
[**get_behavior_policy**](DataHubBehaviorPoliciesApi.md#get_behavior_policy) | **GET** /api/v1/data-hub/behavior-validation/policies/{policyId} | Get a  policy
[**update_behavior_policy**](DataHubBehaviorPoliciesApi.md#update_behavior_policy) | **PUT** /api/v1/data-hub/behavior-validation/policies/{policyId} | Update an existing policy



## create_behavior_policy

> crate::models::BehaviorPolicy create_behavior_policy(behavior_policy)
Create a new policy

Create a behavior policy  This endpoint requires at least HiveMQ version 4.20.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**behavior_policy** | [**BehaviorPolicy**](BehaviorPolicy.md) | The policy that should be created. | [required] |

### Return type

[**crate::models::BehaviorPolicy**](BehaviorPolicy.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_behavior_policy

> delete_behavior_policy(policy_id)
Delete a behavior policy

Deletes an existing policy.    

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_id** | **String** | The identifier of the policy to delete. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_behavior_policies

> crate::models::BehaviorPolicyList get_all_behavior_policies(fields, policy_ids, client_ids, limit, cursor)
Get all policies

Get all policies.    This endpoint returns the content of the policies with the content-type `application/json`.    This endpoint requires at least HiveMQ version 4.20.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, createdAt, lastUpdatedAt, deserialization, matching, behavior, onTransitions |  |
**policy_ids** | Option<**String**> | Comma-separated list of policy ids used for filtering. Multiple filters can be applied together. |  |
**client_ids** | Option<**String**> | Comma-separated list of MQTT client identifiers that are used for filtering. Client identifiers are matched by the retrieved policies. Multiple filters can be applied together. |  |
**limit** | Option<**i32**> | Specifies the page size for the returned results. Has to be between 10 and 500. Default page size is 50. Limit is ignored if the 'topic' query parameter is set. |  |
**cursor** | Option<**String**> | The cursor that has been returned by the previous result page. Do not pass this parameter if you want to fetch the first page. |  |

### Return type

[**crate::models::BehaviorPolicyList**](BehaviorPolicyList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_behavior_policy

> crate::models::BehaviorPolicy get_behavior_policy(policy_id, fields)
Get a  policy

Get a specific policy.    This endpoint returns the content of the policy with the content-type `application/json`.    This endpoint requires at least HiveMQ version 4.20.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_id** | **String** | The identifier of the policy. | [required] |
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, createdAt, lastUpdatedAt, deserialization, matching, behavior, onTransitions |  |

### Return type

[**crate::models::BehaviorPolicy**](BehaviorPolicy.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_behavior_policy

> crate::models::BehaviorPolicy update_behavior_policy(policy_id, behavior_policy)
Update an existing policy

Update a behavior policy  The path parameter 'policyId' must match the 'id' of the policy in the request body.  This endpoint requires at least HiveMQ version 4.20.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_id** | **String** | The identifier of the policy. | [required] |
**behavior_policy** | [**BehaviorPolicy**](BehaviorPolicy.md) | The policy that should be updated. | [required] |

### Return type

[**crate::models::BehaviorPolicy**](BehaviorPolicy.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

