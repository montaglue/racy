use std::{collections::HashMap, fs::File, io::Read};

use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};

use crate::plan_composer::PlanComposer;

pub mod plan_composer;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum MetricType {
    #[serde(rename = "sum")]
    Sum,
    #[serde(rename = "nsTiming")]
    NsTiming,
    #[serde(rename = "timing")]
    Timing,
    #[serde(rename = "size")]
    Size,
    #[serde(rename = "average")]
    Avarage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metric {
    name: String,
    #[serde(rename = "accumulatorId")]
    accumulator_id: i64,
    #[serde(rename = "metricType")]
    metric_type: MetricType,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SparkPlanInfo {
    #[serde(rename = "nodeName")]
    name: String,
    #[serde(rename = "simpleString")]
    simple_string: String,
    metadata: HashMap<String, Value>, // Empty object as far as I seen
    children: Vec<SparkPlanInfo>,
    metrics: Vec<Metric>,
}

impl SparkPlanInfo {
    pub fn leafs(&self) -> Vec<&Self> {
        if self.children.is_empty() {
            return vec![self];
        }
        self.children
            .iter()
            .flat_map(|child| child.leafs())
            .collect()
    }

    /// Converts the SparkPlanInfo tree to Mermaid flowchart markdown
    pub fn to_mermaid(&self) -> String {
        let mut mermaid = String::from("flowchart TD\n");
        let mut node_counter = 0;

        self.build_mermaid_recursive(&mut mermaid, &mut node_counter, None);

        mermaid
    }

    /// Recursive helper function to build the Mermaid diagram
    fn build_mermaid_recursive(
        &self,
        mermaid: &mut String,
        node_counter: &mut usize,
        parent_id: Option<String>,
    ) -> String {
        let current_id = format!("node_{}", node_counter);
        *node_counter += 1;

        // Escape special characters and create node label
        let escaped_name = self.escape_mermaid_text(&self.name);

        // Create a multi-line label with name and simple_string
        let label = escaped_name;

        // Add node definition
        mermaid.push_str(&format!("    {}[\"{}\"]\n", current_id, label));

        // Add edge from parent if exists
        if let Some(parent) = parent_id {
            mermaid.push_str(&format!("    {} --> {}\n", current_id, parent));
        }

        // Process children
        for child in &self.children {
            child.build_mermaid_recursive(mermaid, node_counter, Some(current_id.clone()));
        }

        current_id
    }

    /// Escapes special characters for Mermaid text
    fn escape_mermaid_text(&self, text: &str) -> String {
        text.replace("\"", "&quot;")
            .replace("\\", "\\\\")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "    ")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskMetrics {
    disk_bytes_spilled: i64,
    executor_cpu_time: i64,
    executor_deserialize_cpu_time: i64,
    executor_deserialize_time: i64,
    executor_run_time: i64,
    input_metrics: Value,
    jvm_gc_time: i64,
    memory_bytes_spilled: i64,
    output_metrics: Value,
    peak_execution_memory: i64,
    result_serialization_time: i64,
    result_size: i64,
    shuffle_read_metrics: Value,
    shuffle_write_metrics: Value,
    updated_blocks: Vec<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "Event")]
pub enum SparkEvent {
    SparkListenerLogStart {
        #[serde(rename = "Spark Version")]
        spark_version: String,
    },
    SparkListenerResourceProfileAdded {
        #[serde(rename = "Executor Resource Requests")]
        executor_resource_requests: Value,
        #[serde(rename = "Resource Profile Id")]
        resource_profile_id: i64,
        #[serde(rename = "Task Resource Requests")]
        task_resource_requests: Value,
    },
    SparkListenerBlockManagerAdded {
        #[serde(rename = "Block Manager ID")]
        block_manager_id: Value,
        #[serde(rename = "Maximum Memory")]
        maximum_memory: i64,
        #[serde(rename = "Maximum Offheap Memory")]
        maximum_offheap_memory: i64,
        #[serde(rename = "Maximum Onheap Memory")]
        maximum_onheap_memory: i64,
        #[serde(rename = "Timestamp")]
        timestamp: i64,
    },
    SparkListenerEnvironmentUpdate {
        #[serde(rename = "Classpath Entries")]
        classpath_entries: Value,
        #[serde(rename = "Hadoop Properties")]
        hadoop_properties: Value,
        #[serde(rename = "JVM Information")]
        jvm_information: Value,
        #[serde(rename = "Metrics Properties")]
        metric_properties: Value,
        #[serde(rename = "Spark Properties")]
        spark_properties: Value,
        #[serde(rename = "System Properties")]
        system_properties: Value,
    },
    #[serde(rename = "org.apache.spark.sql.execution.ui.SparkListenerSQLAdaptiveExecutionUpdate")]
    SparkListenerSQLAdaptiveExecutionUpdate {
        #[serde(rename = "executionId")]
        execution_id: i64,
        #[serde(rename = "physicalPlanDescription")]
        physical_plan_description: String,
        #[serde(rename = "sparkPlanInfo")]
        spark_plan_info: SparkPlanInfo,
    },
    SparkListenerTaskEnd {
        #[serde(rename = "Stage Attempt ID")]
        stage_attempt_id: i64,
        #[serde(rename = "Stage ID")]
        stage_id: i64,
        #[serde(rename = "Task End Reason")]
        task_end_reason: Value,
        #[serde(rename = "Task Executor Metrics")]
        task_executor_metrics: Value,
        #[serde(rename = "Task Info")]
        task_info: Value, //TODO: make this special object
        #[serde(rename = "Task Metrics")]
        task_metrics: Value,
        #[serde(rename = "Task Type")]
        task_type: String,
    },
    #[serde(untagged)]
    Unknown(Value),
}

impl SparkEvent {
    pub fn is_spark_listener_sql_adaptive_execution_update(&self) -> bool {
        matches!(
            self,
            SparkEvent::SparkListenerSQLAdaptiveExecutionUpdate { .. }
        )
    }
}

fn get_json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "Null",
        Value::Bool(_) => "Boolean",
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Array(_) => "Array",
        Value::Object(_) => "Object",
    }
}

impl SparkEvent {
    pub fn verify(&self) {
        return;
        loop {
            let SparkEvent::Unknown(value) = self else {
                break;
            };
            let obj = value.as_object().unwrap();
            if obj["Event"].as_str().unwrap() == "SparkListenerTaskEnd" {
                panic!("See you here")
            } else {
                break;
            }
        }

        let SparkEvent::SparkListenerTaskEnd {
            task_metrics: value,
            ..
        } = self
        else {
            return;
        };
        let Some(value) = value.as_object() else {
            unreachable!()
        };

        let keys = value.keys().collect::<Vec<_>>();
        let mut types = Vec::new();
        for key in keys.iter() {
            types.push(get_json_type(&value[*key]));
        }
        println!(
            "Keys are: {:?}",
            keys.into_iter().zip(types.into_iter()).collect::<Vec<_>>()
        );
        let input_metrics = value["Input Metrics"].as_object().unwrap();
        let shuffle_read_metrics = value["Shuffle Read Metrics"].as_object().unwrap();
        let shuffle_write_metrics = value["Shuffle Write Metrics"].as_object().unwrap();
        let output_metrics = value["Output Metrics"].as_object().unwrap();
        println!(
            "Input Metrics keys: {:?}",
            input_metrics
                .keys()
                .map(|k| (k, get_json_type(&input_metrics[k])))
                .collect::<Vec<_>>()
        );
        println!(
            "Shuffle Read Metrics keys: {:?}",
            shuffle_read_metrics
                .keys()
                .map(|k| (k, get_json_type(&shuffle_read_metrics[k])))
                .collect::<Vec<_>>()
        );
        println!(
            "Shuffle Write Metrics keys: {:?}",
            shuffle_write_metrics
                .keys()
                .map(|k| (k, get_json_type(&shuffle_write_metrics[k])))
                .collect::<Vec<_>>()
        );
        println!(
            "Output Metrics keys: {:?}",
            output_metrics
                .keys()
                .map(|k| (k, get_json_type(&output_metrics[k])))
                .collect::<Vec<_>>()
        );
        panic!();
    }
}

fn get_plan(raw: String) -> SparkPlanInfo {
    let objects: Result<Vec<SparkEvent>, _> = raw
        .lines()
        .map(|obj| {
            let mut deser = serde_json::Deserializer::from_str(obj);
            deser.disable_recursion_limit();
            SparkEvent::deserialize(&mut deser).inspect(|value: &SparkEvent| {
                value.verify();
            })
        })
        .collect();
    let objects = objects.unwrap();
    for object in objects.iter() {
        let SparkEvent::SparkListenerSQLAdaptiveExecutionUpdate {
            spark_plan_info, ..
        } = object
        else {
            continue;
        };
        return spark_plan_info.clone();
    }
    unreachable!()
}

fn parse_file(name: String) -> SparkPlanInfo {
    let mut file = File::open(name).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    get_plan(text)
}

fn main() {
    let flarion_plan = parse_file("./flarion_input.json".to_string());
    let spark_plan = parse_file("./spark_input.json".to_string());
    //println!("{:#?}", spark_plan.leafs().into_iter().map(|info| {info.simple_string.clone()}).collect::<Vec<_>>());
    let composer = PlanComposer::new(flarion_plan, spark_plan).unwrap();
    println!("{}", composer.to_mermaid())
}
