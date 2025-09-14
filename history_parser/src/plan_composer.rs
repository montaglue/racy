use std::{mem, rc::Rc};

use crate::{Metric, SparkPlanInfo};

impl Operator {
    pub fn new(plan: SparkPlanInfo) -> Self {
        Self {
            name: plan.name,
            simple_string: plan.simple_string,
            metrics: plan.metrics,
        }
    }
}

pub struct SquishedPlan {
    children: Vec<SquishedPlan>,
    operators: Vec<Operator>,
}

impl SquishedPlan {
    pub fn new(mut plan: SparkPlanInfo) -> Self {
        let mut operators = Vec::new();
        while plan.children.len() == 1 {
            let next_plan = plan.children.pop().unwrap();
            operators.push(Operator::new(plan));
            plan = next_plan
        }

        let children = mem::take(&mut plan.children)
            .into_iter()
            .map(Self::new)
            .collect();
        operators.push(Operator::new(plan));

        Self {
            operators,
            children,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Operator {
    name: String,
    simple_string: String,
    metrics: Vec<Metric>,
}

pub struct PlanComposer {
    left_operators: Vec<Operator>,
    right_operators: Vec<Operator>,
    children: Vec<PlanComposer>,
}

impl PlanComposer {
    pub fn new(left: SparkPlanInfo, right: SparkPlanInfo) -> Option<Self> {
        let left_squished = SquishedPlan::new(left);
        let right_squished = SquishedPlan::new(right);
        Self::from_quished((left_squished, right_squished))
    }

    fn from_quished((left, right): (SquishedPlan, SquishedPlan)) -> Option<Self> {
        if left.children.len() != right.children.len() {
            return None;
        }
        let children = left
            .children
            .into_iter()
            .zip(right.children.into_iter())
            .map(Self::from_quished)
            .collect::<Option<_>>()?;
        Some(Self {
            left_operators: left.operators,
            right_operators: right.operators,
            children,
        })
    }

    /// Converts the PlanComposer tree to Mermaid flowchart markdown
    pub fn to_mermaid(&self) -> String {
        let mut mermaid = String::from("flowchart TD\n    classDef flarion fill:yellow,stroke:#000,color:#000;\n    classDef spark fill:#00008b;\n");
        let mut node_counter = 0;
        let mut subgraph_counter = 0;

        self.build_mermaid_recursive(&mut mermaid, &mut node_counter, &mut subgraph_counter, None);

        mermaid
    }

    fn build_mermaid_recursive(
        &self,
        mermaid: &mut String,
        node_counter: &mut usize,
        subgraph_counter: &mut usize,
        parent_info: Option<(String, String)>, // (last_left_node_id, last_right_node_id)
    ) -> (String, String) {
        let current_subgraph = format!("subgraph_{}", subgraph_counter);
        *subgraph_counter += 1;

        // Start subgraph
        mermaid.push_str(&format!(
            "    subgraph {} [\"Plan {}\"]\n",
            current_subgraph,
            *subgraph_counter - 1
        ));

        // Build left operator chain
        let first_left_node =
            self.build_operator_chain(&self.left_operators, "left", "flarion", mermaid, node_counter);

        // Build right operator chain
        let first_right_node =
            self.build_operator_chain(&self.right_operators, "right", "spark", mermaid, node_counter);

        // End subgraph
        mermaid.push_str("    end\n\n");

        // Get the last nodes of current chains for linking to children
        let last_left_node = self.get_last_node_id(&self.left_operators, &first_left_node);
        let last_right_node = self.get_last_node_id(&self.right_operators, &first_right_node);

        // Link to parent if exists
        if let Some((parent_last_left, parent_last_right)) = parent_info {
            if !first_left_node.is_empty() {
                mermaid.push_str(&format!(
                    "    {} --> {}\n",
                    first_left_node, parent_last_left
                ));
            }
            if !first_right_node.is_empty() {
                mermaid.push_str(&format!(
                    "    {} --> {}\n",
                    first_right_node, parent_last_right
                ));
            }
        }

        // Process children
        for child in &self.children {
            child.build_mermaid_recursive(
                mermaid,
                node_counter,
                subgraph_counter,
                Some((last_left_node.clone(), last_right_node.clone())),
            );
        }

        (last_left_node, last_right_node)
    }

    fn build_operator_chain(
        &self,
        operators: &[Operator],
        chain_type: &str,
        node_class: &str,
        mermaid: &mut String,
        node_counter: &mut usize,
    ) -> String {
        if operators.is_empty() {
            return String::new();
        }

        let mut first_node_id = String::new();
        let mut previous_node_id = String::new();

        for (i, operator) in operators.iter().enumerate() {
            let node_id = format!("{}_{}", chain_type, node_counter);
            *node_counter += 1;

            if i == 0 {
                first_node_id = node_id.clone();
            }

            // Create node label
            let escaped_name = self.escape_mermaid_text(&operator.name);

            let label = escaped_name;

            // Add metrics info if present
            let final_label = if !operator.metrics.is_empty() {
                format!("{}<br/>({} metrics)", label, operator.metrics.len())
            } else {
                label
            };

            // Add node to mermaid
            mermaid.push_str(&format!("        {}[\"{}\"]\n", node_id, final_label));
            mermaid.push_str(&format!("        class {} {}\n", node_id, node_class));


            // Link to previous node in chain
            if !previous_node_id.is_empty() {
                mermaid.push_str(&format!("        {} --> {}\n", node_id, previous_node_id));
            }

            previous_node_id = node_id;
        }

        first_node_id
    }

    fn get_last_node_id(&self, operators: &[Operator], first_node_id: &str) -> String {
        if operators.is_empty() {
            return String::new();
        }

        // Extract the counter from first node and calculate last node
        if let Some(underscore_pos) = first_node_id.rfind('_') {
            if let Ok(first_counter) = first_node_id[underscore_pos + 1..].parse::<usize>() {
                let chain_type = &first_node_id[..underscore_pos];
                let last_counter = first_counter + operators.len() - 1;
                return format!("{}_{}", chain_type, last_counter);
            }
        }

        first_node_id.to_string()
    }

    fn escape_mermaid_text(&self, text: &str) -> String {
        text.replace("\"", "&quot;")
            .replace("\n", "<br/>")
            .replace("\\", "\\\\")
            .replace("\r", "\\r")
            .replace("\t", "    ")
    }
}
