#!/bin/bash

echo "🔧 Fixing PostgreSQL setup..."

# Add PostgreSQL@15 to PATH
export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"

# Check if PostgreSQL is running
if ! brew services list | grep -q "postgresql@15.*started"; then
    echo "Starting PostgreSQL@15..."
    brew services start postgresql@15
    sleep 3
fi

# Create the database
echo "Creating database nbgn_backend..."
createdb nbgn_backend 2>/dev/null && echo "✅ Database created!" || echo "Database might already exist"

# Test connection
if psql -d nbgn_backend -c "SELECT 1;" > /dev/null 2>&1; then
    echo "✅ PostgreSQL is working!"
    
    # Add to shell profile for permanent PATH
    echo ""
    echo "To make PostgreSQL commands available permanently, add this to your ~/.zshrc:"
    echo 'export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"'
    echo ""
    echo "Run: echo 'export PATH=\"/opt/homebrew/opt/postgresql@15/bin:\$PATH\"' >> ~/.zshrc"
else
    echo "❌ Cannot connect to PostgreSQL"
fi