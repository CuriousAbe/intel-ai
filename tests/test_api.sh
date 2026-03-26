#!/bin/bash
set -e

AUTH_FILE="$HOME/.intel-ai/auth.json"

if [ ! -f "$AUTH_FILE" ]; then
  echo "❌ Auth file not found: $AUTH_FILE"
  echo "   Run 'intel-ai login' first."
  exit 1
fi

TOKEN=$(python3 -c "import sys,json; print(json.load(open('$AUTH_FILE'))['access_token'])" 2>/dev/null)
if [ -z "$TOKEN" ]; then
  echo "❌ Could not read access_token from $AUTH_FILE"
  exit 1
fi

ACCOUNT_ID=$(echo "$TOKEN" | cut -d'.' -f2 | base64 -d 2>/dev/null | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    auth = d.get('https://api.openai.com/auth', {})
    print(auth.get('chatgpt_account_id') or d.get('chatgpt_account_id', ''))
except Exception:
    print('')
" 2>/dev/null)

echo "Testing API call..."
echo "  Account ID: ${ACCOUNT_ID:-<not found>}"

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "https://chatgpt.com/backend-api/codex/responses" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  ${ACCOUNT_ID:+-H "openai-account-id: $ACCOUNT_ID"} \
  -d '{
    "model": "gpt-5.3-codex",
    "instructions": "Reply with just: OK",
    "input": [{"role": "user", "content": "test"}],
    "store": false,
    "stream": true
  }')

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" == "200" ]; then
  echo "✅ API call successful (200)"
  echo "$BODY" | python3 -m json.tool 2>/dev/null | head -20
else
  echo "❌ API call failed ($HTTP_CODE)"
  echo "$BODY"
  exit 1
fi
