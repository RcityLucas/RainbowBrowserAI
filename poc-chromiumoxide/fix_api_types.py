#!/usr/bin/env python3
import re

# Read the file
with open('src/api/mod.rs', 'r') as f:
    content = f.read()

# Pattern to match .map().map_err() chains
pattern = r'(\s+)(browser\.[a-z_]+\([^)]*\)\.await)\s*\n\s*\.map\(([^}]+)\}\)\)\s*\n\s*\.map_err\(\|e\| anyhow::anyhow!\(e\)\)'

def replace_func(match):
    indent = match.group(1)
    call = match.group(2)
    map_content = match.group(3)
    
    # Clean up the map content
    map_content = map_content.replace('\n', ' ')
    
    return f'''{indent}match {call} {{
{indent}    Ok(_) => Ok({map_content}}})),
{indent}    Err(e) => Err(anyhow::anyhow!(e))
{indent}}}'''

# Apply the replacement
content = re.sub(pattern, replace_func, content)

# Write back
with open('src/api/mod.rs', 'w') as f:
    f.write(content)

print("Fixed type annotation errors in api/mod.rs")