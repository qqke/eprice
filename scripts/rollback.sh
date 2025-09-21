#!/bin/bash

# Rollback script for EPrice application

set -e

DEPLOY_ENV=${1:-production}
ROLLBACK_VERSION=${2}

echo "Starting rollback for $DEPLOY_ENV environment..."

if [[ -z "$ROLLBACK_VERSION" ]]; then
    echo "Error: Rollback version not specified"
    echo "Usage: $0 <environment> <version>"
    exit 1
fi

# Validate environment
if [[ "$DEPLOY_ENV" != "staging" && "$DEPLOY_ENV" != "production" ]]; then
    echo "Error: Invalid environment. Use 'staging' or 'production'"
    exit 1
fi

# Confirm rollback
read -p "Are you sure you want to rollback $DEPLOY_ENV to version $ROLLBACK_VERSION? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Rollback cancelled"
    exit 1
fi

# Set docker-compose configuration
if [[ "$DEPLOY_ENV" == "staging" ]]; then
    export COMPOSE_FILE="docker-compose.yml:docker-compose.staging.yml"
else
    export COMPOSE_FILE="docker-compose.yml:docker-compose.production.yml"
fi

# Pull rollback images
echo "Pulling rollback images..."
docker pull eprice/backend:$ROLLBACK_VERSION
docker pull eprice/frontend:$ROLLBACK_VERSION

# Update image tags in environment
export BACKEND_VERSION=$ROLLBACK_VERSION
export FRONTEND_VERSION=$ROLLBACK_VERSION

# Perform rollback
echo "Rolling back to version $ROLLBACK_VERSION..."
docker-compose up -d

# Wait for containers to be ready
sleep 30

# Verify rollback
echo "Verifying rollback..."
if ! docker-compose exec frontend wget -q --spider http://localhost/; then
    echo "Frontend health check failed after rollback"
    exit 1
fi

if ! docker-compose exec backend curl -f http://localhost:8080/health; then
    echo "Backend health check failed after rollback"
    exit 1
fi

echo "Rollback to version $ROLLBACK_VERSION completed successfully!"

# Send notification
if [[ -n "$SLACK_WEBHOOK_URL" ]]; then
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"EPrice $DEPLOY_ENV rolled back to version $ROLLBACK_VERSION\"}" \
        $SLACK_WEBHOOK_URL
fi