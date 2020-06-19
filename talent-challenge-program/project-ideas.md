## Project ideas

Project maintainers and mentors, please submit the ideas below section using the template. Project ideas selected in season 1 will be listed in [Selected Projects](selected-projects.md) page

### Template

```
#### TiDB Ecosystem Project Name
##### Title
- Description:
- Recommended Skills:
- Mentor(s):
- Upstream Issue or RFC (URL):
```

### Proposed Project ideas

#### BR

##### BR HTTP Storage
- Description: Support HTTP(S) server as source and destination for BR, and allow BR itself act as an authenticated HTTP(S) server to simplify deployment.
- Recommended Skills: Go, Rust, HTTP communication, TLS handling.
- Mentor(s): kennytm
- Upstream Issue or RFC (URL): 
   - https://github.com/pingcap/br/issues/308,
    - https://github.com/pingcap/br/issues/212 

##### BR Export
- Description: Building on top of BR backup, implement an “export” function to produce CSV and SQL dump.
- Recommended Skills: Rust, Go, Row encoding (MVCC and TiDB), gRPC.
- Mentor(s): kennytm
- Upstream Issue or RFC (URL): https://github.com/pingcap/br/issues/351 

#### TiUP Bench

##### Generate BR Backup Archive
- Description: Currently TPC-C/TPC-H data prepared by tiup bench can either be inserted via SQL or dumped into CSV files for bulk ingestion. Both methods are slow compared with BR restore. In this project, we want to directly generate a BR backup archive, so benchmarks not caring about the prepare step could be ramped up quickly.
- Recommended Skills: Go, Row encoding (TiDB), TiKV/RocksDB SST format
- Mentor(s): kennytm
- Upstream Issue or RFC (URL): https://github.com/pingcap/go-tpc/issues/46 

#### TiCDC

##### TiCDC Cloud Storage
- Description: Some usage scenarios need to send the changed events to low-cost storage medium such as S3, in order to keep data for a long time and can consume historical changed events asynchronously. We should design and implement the storage strategy for TiCDC open protocol in cloud storage, and provide the consumption strategies based on the storage sketch in cloud storage.
- Recommended Skills: Go, Cloud storage service.
- Mentor(s): yangfei
Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/655

##### TiCDC Snapshot Level Consistency Replication
* Description: In many scenarios users want to ensure the downstream is replicated to a globally consistent state, as TiCDC supports eventual transaction consistency, and TiDB supports snapshot read, we could combine these two features and provide a snapshot level consistency replication strategy.
* Recommended Skills: Go, Transaction.
* Mentor(s): yangfei
* Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/658

##### TiCDC New Mechanism for ResolvedTS
* Description: TiCDC needs a timestamp (called ResolvedTS) in which we can ensure all transactions which start before this timestamp have completed and been sent from TiKV to TiCDC. For now, TiKV must advance the ResolvedTS in the leader of the Raft group. We require a new mechanism to eliminate this restriction.
* Recommended Skills: Go, Rust, Raft
* Mentor(s): tangminghua
* Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/657

##### TiCDC Supports Avro Sink and Kafka Connector
* Description: Apache Kafka provides a flexible connectors mechanism, which is widely used in change data capture scenarios. We want to implement an Avro sink and make TiCDC compatible with the Kafka connector ecosystem.
* Recommended Skills: Go, Kafka.
* Mentor(s): yangfei, liuzixiong
* Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/660

##### TiCDC Support a status mechanism for changefeed
* Description: Various errors may be encountered during the execution of replication tasks (e.g. downstream link failure, incompatible DDLs, etc.). we hope to provide a mechanism so that users can quickly understand the status of the current replication task (normal or abnormal), and the reason for the error.
* Recommended Skills: Go, SQL.
* Mentor(s): zhaoyilin
* Upstream Issue or RFC (URL): https://github.com/pingcap/ticdc/issues/664

#### PD

##### PD Store & Region UI
* Description: Add display and operation UI for PD stores and regions, in order to reduce the necessity of using pd-ctl and enhance user experience.
* Recommended Skills: Go, Web Frontend.
* Mentor(s): @HundunDM @breeswish
* Upstream Issue or RFC (URL): https://docs.google.com/document/d/1moQVhvIgqu_FWuv_UMB76AM5tETJkyeWnzI04wyevhk/edit

##### PD Encryption at rest
* Description: Encryption at rest means that data is encrypted when it is stored, TiKV is already supported, but PD is not. PD stores the meta information of the cluster, especially the Key information of the Region, we need to encrypt it. Here it is proposed that PD also supports encryption.
* Recommended Skills: Go, Cryptography
* Mentor(s)：@yiwu-arbug
* Upstream Issue or RFC (URL): 
	* TiKV: https://github.com/tikv/rfcs/blob/929bf1f5d675b555c013d863599544afd9bfe812/text/2020-04-27-encryption-at-rest.md
	* PD: WIP

#### TiDB

##### Live Execution Plan
* Description: Provide a way to query current execution details for running SQLs. This feature is similar to the Live Query Statistics in the SQL server. SQL server allows viewing the live execution plan for any queries.
* Recommended Skills: Go, SQL
* Mentor(s): @crazycs520 @breeswish @qw4990
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/17692

##### Print Txn ID and Query ID in log to trace the whole lifetime of a Txn / SQL
* Description: TiDB now outputs TxnStartTs or ConnId in logs, which is not sufficient. This task is to allocate and print a unique TxnId for each transaction, as well as a unique QueryId for each query, for both TiDB and TiKV.
* Recommended Skills: Go, Rust
* Mentor(s): @crazycs520 @breeswish @SunRunAway
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/17845

##### Defining Placement Rules in SQL
* Description: TiDB supports placement rules, but it can only be defined in configuration files. If there is an approach to configure placement rules through statements, usability can be improved. 
* Recommended Skills: Go, Data Definition Language
* Mentor(s): @djshow832
* Upstream Issue or RFC (URL): 
	* https://github.com/pingcap/tidb/issues/18030
	* https://docs.google.com/document/d/18Kdhi90dv33muF9k_VAIccNLeGf-DdQyUc8JlWF9Gok/edit#

##### Support Multiple Tables Rename in A Statement
* Description: TiDB supports renaming one table but doesn’t support renaming multiple tables. This task wants to support renaming multiple tables in a statement.
* Recommended Skills: Go, DDL
* Mentor(s): @zimulala
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/14766

##### Support the operation of dropping multi-indexes in a statement
* Description: This task wants to support the operation of dropping multi-indexes in a statement.
* Recommended Skills: Go, DDL
* Mentor(s): @zimulala
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/14765

##### Dropping column with index covered
* Description: Support the operation of deleting columns containing indexes.
* Recommended Skills: Go, DDL
* Mentor(s): @zimulala
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/3364

##### Support SAVEPOINT in TiDB
* Description: SAVEPOINT is a feature basically supported by mainstream traditional databases (Oracle, DB2, MySQL, and PostgreSQL), and its role is to partially roll back multiple statements of the transaction being executed. TiDB does not currently support this feature.
* Recommended Skills: Go
* Mentor(s): @bobotu
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/6840

##### Support LIST COLUMNS partitioning
* Description: LIST COLUMNS partitioning is a feature basically supported by MySQL 8.0, and its role is to define table partition with list columns. TiDB does not currently support this feature.
* Recommended Skills: Go, DDL, SQL
* Mentor(s): @imtbkcat
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/18052

##### Support Global Index for Partition Table
* Description: Currently TiDB only supports local partitioned indexes that compatible with mysql，but other database(e.g. Oracle) also supports global partitioned indexes
* Recommended Skills: Go, DDL, SQL
* Mentor(s): @tiancaiamao
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/18032

##### Implement Join Order Hint
* Description: Provide a comment style SQL hint to specify the order of joins in the query.
* Recommended Skills: Go, SQL
* Mentor(s): @eurekaka
* Upstream Issue or RFC (URL): WIP

##### Reduce memory consumption of stats data
* Description: When there're lots of tables in a TiDB cluster, caching all the stats data into a single TiDB server may cause a high memory consumption when the TiDB server bootstrapped. It increases the OOM risk of the TiDB server.
* Recommended Skills: Go
* Mentor(s): @SunRunAway
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/16572

##### Reduce memory consumption of table meta
* Description: TiDB always loads all tables in all schema at once in bootstrap, which can consume very much memory. It increases the OOM risk of the TiDB server.
* Recommended Skills: Go
* Mentor(s): @bb7133, @SunRunAway
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/16572

##### Support utf8_unicode_ci/utf8mb4_unicode_ci collation
* Description: Implement the utf8_unicode_ci/utf8mb4_unicode_ci collation algorithm and optimize them.
* Recommended Skills: Go
* Mentor(s): @wjhuang2016
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/17596

##### Implement More Diagnostics Rules 
* Description: Add more diagnose rule in TiDB SQL diagnose. 
* Recommended Skills: Go, SQL
* Mentor(s): @crazycs520
* Upstream Issue or RFC (URL): https://github.com/pingcap/tidb/issues/17927

#### TiKV

##### Witness
* Description: support raft witness role, and use it in the dual data center deployment scenario, TiDB can still provide service no matter which one of the data center crash
* Recommended Skills: Rust, Raft
* Mentor(s): @busyjay
* Upstream Issue or RFC(URL): WIP

##### Loose Follower Read
* Description: Provide a way to read follower doesn’t need to send a request to the leader
* Recommended Skills: Rust, Raft
* Mentor(s): @hicqu
* Upstream Issue or RFC (URL): WIP

##### Write flow control
* Description: Control the writing flow by requests' latency, and make the writing smoothly
* Recommended Skills: Rust
* Mentor(s): @Conner1996
* Upstream Issue or RFC (URL): https://docs.google.com/document/d/1rgm4rS2youwJpy_zpC39BJgxPpnwk7DeuF5LjvWrBZ8/edit#

##### SQL Statement Level Statistics
* Description: To make it possible to know what statement is the root cause of hot regions, we need to add local statistics to TiKV that counts top K hot statements and then reports them to PD.
* Recommended Skills: Rust, Go
* Mentor(s): @breeswish @HundunDM
* Upstream Issue or RFC (URL): https://github.com/pingcap-incubator/tidb-dashboard/issues/574
