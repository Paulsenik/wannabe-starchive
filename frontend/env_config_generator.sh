#!/bin/sh

# Generate env-config.js with environment variables
cat > /usr/share/nginx/html/env-config.js << EOF
window.ENV_CONFIG = {
  API_BASE_URL: "${BACKEND_URL:-http://localhost:8000}",
  APP_NAME: "${APP_NAME:-Paulsenik's StarCitizen Content Search}",
  DEBUG_MODE: "${DEBUG_MODE:-false}"
};
EOF

echo "Environment configuration generated:"
cat /usr/share/nginx/html/env-config.js