#!/bin/bash

# Validate GitHub Actions workflows

set -e

echo "🔍 Validating GitHub Actions workflows..."

# Check if workflows directory exists
if [ ! -d ".github/workflows" ]; then
    echo "❌ .github/workflows directory not found"
    exit 1
fi

# Check each workflow file
for workflow in .github/workflows/*.yml; do
    if [ -f "$workflow" ]; then
        echo "✅ Found workflow: $workflow"

        # Basic YAML syntax check
        if python3 -c "import yaml; yaml.safe_load(open('$workflow'))" 2>/dev/null; then
            echo "   ✅ YAML syntax is valid"
        else
            echo "   ❌ YAML syntax error in $workflow"
            exit 1
        fi
    fi
done

# Check for deprecated actions
echo "🔍 Checking for deprecated actions..."

# List of deprecated actions to check
DEPRECATED_ACTIONS=(
    "actions/upload-artifact@v3"
    "actions/cache@v3"
    "actions/create-release@v1"
    "actions/upload-release-asset@v1"
)

for action in "${DEPRECATED_ACTIONS[@]}"; do
    if grep -r "$action" .github/workflows/ > /dev/null; then
        echo "❌ Found deprecated action: $action"
        exit 1
    fi
done

echo "✅ All workflows use modern actions"

# Check workflow structure
echo "🔍 Checking workflow structure..."

for workflow in .github/workflows/*.yml; do
    echo "📋 Workflow: $(basename $workflow)"

    # Check for required fields
    if ! grep -q "name:" "$workflow"; then
        echo "   ❌ Missing 'name' field"
        exit 1
    fi

    if ! grep -q "on:" "$workflow"; then
        echo "   ❌ Missing 'on' field"
        exit 1
    fi

    if ! grep -q "jobs:" "$workflow"; then
        echo "   ❌ Missing 'jobs' field"
        exit 1
    fi

    echo "   ✅ Basic structure is valid"
done

echo ""
echo "🎉 All GitHub Actions workflows are valid and use modern versions!"
echo ""
echo "📋 Summary of workflows:"
echo "   - ci.yml: Continuous integration with testing and building"
echo "   - release.yml: Automated releases for multiple platforms"