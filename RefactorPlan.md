# graphql处理器重构计划（设计）



## Schema



### 派生/实现

* `Clone`

* `Debug`

### 结构

| 字段         | 类型                     | 访问性       | 说明                        |
| ------------ | ------------------------ | ------------ | --------------------------- |
| id           | `String`                 | crate public | schema的id                  |
| queries      | `QueryMap`               | crate public | query的字段/接口信息        |
| mutations    | `Option<MutationMap>`    | crate public | mutation的字段/接口信息     |
| subscritions | `Option<SubscritionMap>` | crate public | subscription的字段/接口信息 |
| type_storage | `TypeStorage`            | crate public | 存放类型信息的存储器        |

###  方法 

#### group_document

* 访问性：private
* 说明：整理请求文档
* 参数 

| 字段  | 类型       | 说明       |
| ----- | ---------- | ---------- |
| &self |            |            |
| doc   | `Document` | 请求的文档 |

* 返回

`Result<OperationGroup>`：规整后的请求文档信息



#### execute_document

* 访问性：private
* 说明：根据要执行的operation执行文档
* 参数

| 字段           | 类型             | 说明                  |
| -------------- | ---------------- | --------------------- |
| &self          |                  |                       |
| context        | `QLContext`      | 执行时的上下文        |
| doc            | `Document`       | 请求的文档            |
| operation_name | `Option<String>` | 要执行的operation名称 |

* 返回

`Result<ResultValue>`：执行结果 



#### execute_selection_set

* 访问性：private
* 说明：执行匿名的selection set
* 参数

| 字段      | 类型                                   | 说明                      |
| --------- | -------------------------------------- | ------------------------- |
| &self     |                                        |                           |
| context   | `QLContext`                            | 执行时的上下文            |
| sets      | `SelectionSet`                         | 请求中的SelectionSet的AST |
| fragments | `&HashMap<String, FragmentDefinition>` | 请求中的片段Map           |

* 返回

`Result<ResultValue>`：执行结果 



#### execute_query

* 访问性：private
* 说明：执行query
* 参数

| 字段      | 类型                                   | 说明               |
| --------- | -------------------------------------- | ------------------ |
| &self     |                                        |                    |
| context   | `QLContext`                            | 执行时的上下文     |
| query     | `AstQuery`                             | 请求中的Query的AST |
| fragments | `&HashMap<String, FragmentDefinition>` | 请求中的片段Map    |

* 返回

`Result<ResultValue>`：执行结果 



#### execute_mutation

* 访问性：private
* 说明：执行mutation
* 参数

| 字段      | 类型                                   | 说明                  |
| --------- | -------------------------------------- | --------------------- |
| &self     |                                        |                       |
| context   | `QLContext`                            | 执行时的上下文        |
| mutation  | `AstMutation`                          | 请求中的Mutation的AST |
| fragments | `&HashMap<String, FragmentDefinition>` | 请求中的片段Map       |

* 返回

`Result<ResultValue>`：执行结果 

 

### 实现

* `Drop`



## OperationGroup

### 派生/实现

### 结构

| 字段          | 类型                                     | 访问性  | 说明                           |
| ------------- | ---------------------------------------- | ------- | ------------------------------ |
| selection_set | `Option<SelectionSet>`                   | private | 请求的匿名query，selection set |
| queries       | `HashMap<OperationKey, AstQuery>`        | private | 请求的query组                  |
| mutations     | `HashMap<OperationKey, AstMutation>`     | private | 请求的mutation组               |
| subscriptions | `HashMap<OperationKey, AstSubscription>` | private | 请求的subscription组           |
| fragments     | `HashMap<String, FragmentDefinition>`    | private | 请求时定义的片段               |

###  方法

#### count

#### contains_anonymous

###  实现

* `Default`




## OperationKey (enum)

* `Anonymous`
* `RealNamed(String)`

###  实现

* `ToString`
* `From<Option<String>>`


