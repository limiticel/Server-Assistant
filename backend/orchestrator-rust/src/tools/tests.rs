#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::config::Settings;
    use crate::tools::build_registry;

    #[test]
    fn tools_health_reports_registered_tools() {
        let settings = Settings::from_env();
        let registry = build_registry(&settings);
        let tool = registry
            .find_for_role("tools_health", "admin")
            .expect("tools_health should be registered");

        let result = (tool.handler)(&json!({}), &registry).expect("tool should run");

        assert_eq!(result["status"], "ok");
        assert!(result["tools_count"].as_u64().unwrap_or_default() >= 1);
        assert!(result["tools"]
            .as_array()
            .unwrap()
            .contains(&json!("tools_health")));
    }

    #[test]
    fn sales_role_does_not_see_dev_only_tool() {
        let settings = Settings::from_env();
        let registry = build_registry(&settings);

        assert!(registry
            .find_for_role("gerar_scaffold_api", "sales")
            .is_none());
        assert!(registry
            .find_for_role("gerar_scaffold_api", "dev")
            .is_some());
    }

    #[test]
    fn buscar_cliente_echoes_query() {
        let settings = Settings::from_env();
        let registry = build_registry(&settings);
        let tool = registry
            .find_for_role("buscar_cliente", "sales")
            .expect("buscar_cliente should be visible for sales");

        let result = (tool.handler)(&json!({ "query": "Empresa" }), &registry)
            .expect("buscar_cliente should run");

        assert_eq!(result["query"], "Empresa");
        assert_eq!(result["total"], 1);
    }
}
