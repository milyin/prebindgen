#!/usr/bin/env python3

import re

# Read the file
with open('/Users/milyin/ZS/prebindgen/prebindgen/src/codegen/tests.rs', 'r') as f:
    content = f.read()

# Replace all instances of default_test_context with default_test_codegen and context with codegen
content = content.replace('default_test_context', 'default_test_codegen')
content = content.replace('let _context =', 'let _codegen =')
content = content.replace('let context =', 'let codegen =')

# Replace all transform_function_to_stub calls to use method syntax
# Pattern: transform_function_to_stub(file, &context, ...)
# Replace with: codegen.transform_function_to_stub(file, ...)
content = re.sub(
    r'transform_function_to_stub\(\s*([^,]+),\s*&context,\s*([^,]+),\s*([^)]+)\)',
    r'codegen.transform_function_to_stub(\1, \2, \3)',
    content
)

# Also handle the case where there might be &codegen instead of &context
content = re.sub(
    r'transform_function_to_stub\(\s*([^,]+),\s*&codegen,\s*([^,]+),\s*([^)]+)\)',
    r'codegen.transform_function_to_stub(\1, \2, \3)',
    content
)

# Write the file back
with open('/Users/milyin/ZS/prebindgen/prebindgen/src/codegen/tests.rs', 'w') as f:
    f.write(content)

print("Updated test file successfully")
