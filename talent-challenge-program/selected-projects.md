## Selected Projects in Season 1

Selected Projects in Season 1 are listed below. Mentee candidates who want to participate in a certain project please refer to [Mentee Selection Process](README.md#mentees)

The difficulty of the project is divided into three levels: Very hard, Hard, and Medium, the corresponding pre-tax bonus is RMB 10000 to a very hard project, RMB 8000 to a hard project and RMB 5000 to a medium project respectively. 

### Template

```
#### TiDB Ecosystem Project Name
##### Title
- Description:
- Recommended Skills:
- Mentor(s):
- Upstream Issue or RFC (URL):
- Difficulty:
```

## List of Selected Projects

#### BR

##### BR Export

- Description: Building on top of BR backup, implement an “export” function to produce CSV and SQL dump.
- Recommended Skills: Rust, Go, Row encoding (MVCC and TiDB), gRPC.
- Mentor(s): @kennytm
- Upstream Issue or RFC (URL): https://github.com/pingcap/br/issues/351
- Difficulty: Hard

#### TiCDC

##### TiCDC Cloud Storage

* Description: Some usage scenarios need to send the changed events to low-cost storage medium such as S3, in order to keep data for a long time and enable consuming historical changed events asynchronously. We should design and implement the storage strategy for TiCDC open protocol in cloud storage, and provide the consumption strategies based on the storage sketch in cloud storage.
* Recommended Skills: Go, Cloud storage service.
* Mentor(s): @amyangfei
* Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/655
* Difficulty: Medium

##### TiCDC Snapshot Level Consistency Replication

* Description: In many scenarios users want to ensure the downstream is replicated to a globally consistent state, as TiCDC supports eventual transaction consistency, and TiDB supports snapshot read, we could combine these two features and provide a snapshot level consistency replication strategy.
* Recommended Skills: Go, Transaction.
* Mentor(s): @amyangfei
* Mentee: [@Colins110](https://github.com/Colins110)
* Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/658
* Difficulty: Medium


##### TiCDC Supports Avro Sink and Kafka Connector

* Description: Apache Kafka provides a flexible connectors mechanism, which is widely used in change data capture scenarios. We want to implement an Avro sink and make TiCDC compatible with the Kafka connector ecosystem.
* Recommended Skills: Go, Kafka.
* Mentor(s): @amyangfei, @liuzx
* Mentee: [@qinggniq](https://github.com/qinggniq)
* Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/660
* Difficulty: Medium

#### TiDB

##### Live Execution Plan

* Description: Provide a way to query current execution details for running SQLs. This feature is similar to the Live Query Statistics in the SQL server. SQL server allows viewing the live execution plan for any queries.
* Recommended Skills: Go, SQL
* Mentor(s): @crazycs520 @breeswish @qw4990
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/17692
* Difficulty: Medium

##### Defining Placement Rules in SQL

* Description: TiDB supports placement rules, but it can only be defined in configuration files. If there is an approach to configure placement rules through statements, usability can be improved. 
* Recommended Skills: Go, Data Definition Language
* Mentor(s): @djshow832
* Mentee: [@xhe](https://github.com/xhebox)
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/18030
* Difficulty: Hard

##### Support SAVEPOINT in TiDB

* Description: SAVEPOINT is a feature basically supported by mainstream traditional databases (Oracle, DB2, MySQL, and PostgreSQL), and its role is to partially roll back multiple statements of the transaction being executed. TiDB does not currently support this feature.
* Recommended Skills: Go
* Mentor(s): @bobotu
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/6840
* Difficulty: Very Hard

##### Support Global Index for Partition Table

* Description: Currently TiDB only supports local partitioned indexes that compatible with mysql，but other database(e.g. Oracle) also supports global partitioned indexes
* Recommended Skills: Go, DDL, SQL
* Mentor(s): @tiancaiamao
* Mentee: [@ldeng-ustc](https://github.com/ldeng-ustc)
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/18032
* Difficulty: Hard

##### Reduce memory consumption of stats data

* Description: When there're lots of tables in a TiDB cluster, caching all the stats data into a single TiDB server may cause a high memory consumption when the TiDB server bootstrapped. It increases the OOM risk of the TiDB server.
* Recommended Skills: Go
* Mentor(s): @SunRunAway
* Mentee: @[miamiaoxyz](https://github.com/miamia0)
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/16572
* Difficulty: Medium

##### Support utf8_unicode_ci/utf8mb4_unicode_ci collation

* Description: Implement the utf8_unicode_ci/utf8mb4_unicode_ci collation algorithm and optimize them.
* Recommended Skills: Go, C++, Rust
* Mentor(s): @wjhuang2016
* Mentee: @[xiongjiwei](https://github.com/xiongjiwei)
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/17596
* Difficulty: Medium

#### TiKV

##### SQL Statement Level Statistics

* Description: To make it possible to know what statement is the root cause of hot regions, we need to add local statistics to TiKV that counts top K hot statements and then reports them to PD.
* Recommended Skills: Rust, Go,  Algorithm / Data Structure
* Mentor(s): @breeswish @HundunDM
* Upstream Issue or RFC (URL): https://github.com/pingcap-incubator/tidb-dashboard/issues/574
* Difficulty: Hard

##### HBase protocol support on Zetta

* Description: Zetta is designed as a BigTable implementation on top of TiKV to support structured schemaless data model. In order to elaborate existing HBase ecosystem, we plan to implement the HBase protocol adaptor for Zetta. To achieve this, we need to:
	1. Implement a mock ZooKeeper to eliminate external dependency
	2. Implement RPC protocol of HBase RegionServer
	3. Implement HBase functionality on top of Zetta
* Recommended Skills: Go, Java, HBase RPC
* Mentor(s): pseudocodes, baiyuqing
* Mentee: [@BowenXiao1999](https://github.com/BowenXiao1999)
* Upstream Issue or RFC (URL): https://github.com/zhihu/zetta/issues/2（in Chinese）
* Difficulty: Hard 

#### Chaos Mesh 

##### Support using plugins to extend scheduler  

* Description: In many scenarios users need to customize selector. For example, when our tidb cluster has multiple pds, we just want to inject the failure to the leader. So we need to provide a plugin to let the users to extend the selector, and users can implement their own selector through code. For example, we can create a file called `pd-selector.go` and set it in the selector field.  
* Recommended Skills: Go, Kubernetes
* Mentor(s): @cwen0
* Upstream Issue or RFC (URL): https://github.com/pingcap/chaos-mesh/issues/193
* Difficulty: Medium 

##### Support injecting HTTP fault

* Description: In many scenarios, users need to inject an HTTP delay fault or an HTTP abort fault on the special application. For example, we just want to inject a delay fault on getting `/api/xxx` router.
* Recommended Skills: Go, Kuberenetes 
* Mentor(s): @Yisaer
* Upstream Issue or RFC (URL): https://github.com/pingcap/chaos-mesh/issues/651
* Difficulty: Hard  
