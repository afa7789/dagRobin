#!/bin/bash
set -e

DB="./test-dagrobin.db"
export DAGRDB="$DB"

rm -f "$DB"

echo "=== dagRobin Full Command Flow Test ==="
echo ""

echo "1. Adding tasks..."
./target/release/dagRobin add setup-db "Setup the database" --priority 1 --tags "devops,db" --files "db/schema.sql,db/config.toml"
./target/release/dagRobin add build-api "Build the API" --deps setup-db --priority 2 --tags "backend" --files "src/api.rs,src/handlers.rs"
./target/release/dagRobin add write-tests "Write tests" --deps build-api --priority 3 --tags "testing" --files "tests/api_test.rs"
./target/release/dagRobin add add-docs "Add documentation" --priority 4 --tags "docs" --files "README.md"
echo "   Added 4 tasks"

echo ""
echo "2. Listing all tasks..."
./target/release/dagRobin list --format table

echo ""
echo "3. Getting specific task (setup-db)..."
./target/release/dagRobin get setup-db

echo ""
echo "4. Checking ready tasks..."
./target/release/dagRobin ready --format yaml

echo ""
echo "5. Checking blocked tasks..."
./target/release/dagRobin blocked --format yaml

echo ""
echo "6. Checking if specific task is ready (setup-db)..."
./target/release/dagRobin check setup-db && echo "   setup-db is ready!"

echo ""
echo "7. Checking if specific task is ready (build-api - should be blocked)..."
./target/release/dagRobin check build-api || echo "   build-api is blocked (expected)"

echo ""
echo "8. Claiming a task..."
./target/release/dagRobin claim setup-db --agent test-agent

echo ""
echo "9. Updating task status..."
./target/release/dagRobin update setup-db --status done

echo ""
echo "10. Now checking ready tasks (build-api should be ready)..."
./target/release/dagRobin ready --format yaml

echo ""
echo "11. Claiming build-api..."
./target/release/dagRobin claim build-api --agent test-agent

echo ""
echo "12. Updating with metadata..."
./target/release/dagRobin update build-api --metadata "started:2024-01-01;notes:Building API endpoints"

echo ""
echo "13. Graph (ASCII)..."
./target/release/dagRobin graph

echo ""
echo "14. Graph (Mermaid)..."
./target/release/dagRobin graph --format mermaid

echo ""
echo "15. Graph (DOT)..."
./target/release/dagRobin graph --format dot --output test-graph.dot
cat test-graph.dot

echo ""
echo "16. Exporting tasks..."
./target/release/dagRobin export exported-tasks.yaml

echo ""
echo "17. Importing tasks (merge mode - first delete some tasks)..."
./target/release/dagRobin delete add-docs --force
./target/release/dagRobin import exported-tasks.yaml

echo ""
echo "18. Listing after import..."
./target/release/dagRobin list --format table

echo ""
echo "19. Conflicts detection..."
./target/release/dagRobin conflicts --ready-only --format yaml

echo ""
echo "20. Delete remaining tasks..."
./target/release/dagRobin delete write-tests --force
./target/release/dagRobin delete add-docs --force

echo ""
echo "21. Final list..."
./target/release/dagRobin list --format table

rm -f "$DB" test-graph.dot exported-tasks.yaml

echo ""
echo "=== All tests passed! ==="
