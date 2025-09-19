#!/bin/bash
# Fix all the .map().map_err() patterns in api/mod.rs

sed -i '658,661s/browser\.click(selector)\.await.*\.map_err(|e| anyhow::anyhow!(e))/match browser.click(selector).await {\
                            Ok(_) => Ok(serde_json::json!({"status": "clicked", "selector": selector})),\
                            Err(e) => Err(anyhow::anyhow!(e))\
                        }/' src/api/mod.rs

sed -i '670,672s/browser\.type_text(selector, text)\.await.*\.map_err(|e| anyhow::anyhow!(e))/match browser.type_text(selector, text).await {\
                            Ok(_) => Ok(serde_json::json!({"status": "typed", "selector": selector, "text": text})),\
                            Err(e) => Err(anyhow::anyhow!(e))\
                        }/' src/api/mod.rs

echo "Fixed map errors in api/mod.rs"