use serde_json::{Map, Value, json};

const URL: &str = "http://127.0.0.1:8090/waapi";

pub fn call(uri: &str, args: Option<Value>, options: Option<Value>) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let data = json!({
        "uri": uri,
        "options": options.unwrap_or(json!({})),
        "args": args.unwrap_or(json!({}))
    });
    
    let response = ureq::post(URL)
        .header("content-type", "application/json")
        .send_json(data)
        .map_err(|e| {
            eprintln!("消息发送失败: {}", e);
            e
        })?;
    
    let result: Value = response
        .into_body()
        .read_json()
        .map_err(|e| {
            eprintln!("消息读取失败: {}", e);
            e
        })?;
    
    result.as_object()
        .cloned()
        .ok_or_else(|| "返回的结果不是 JSON 对象".into())
}

pub fn waql_query(query: &str, options: Option<Value>) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let args = json!({
        "waql": query
    });
    call("ak.wwise.core.object.get", Some(args), options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call() {
        let uri = "ak.wwise.core.getInfo";
        let result = call(uri, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_waql_query() {
        let query = "$ from type Event";
        let result = waql_query(query, None);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
