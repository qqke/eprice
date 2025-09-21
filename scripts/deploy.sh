#!/bin/bash

# Production deployment script for EPrice application

set -e

# Configuration
DEPLOY_ENV=${1:-production}
VERSION=${2:-latest}
DOCKER_REGISTRY=${DOCKER_REGISTRY:-eprice}

echo "Starting deployment to $DEPLOY_ENV environment..."

# Validate environment
if [[ "$DEPLOY_ENV" != "staging" && "$DEPLOY_ENV" != "production" ]]; then
    echo "Error: Invalid environment. Use 'staging' or 'production'"
    exit 1
fi

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "Error: Docker is not running"
    exit 1
fi

# Pull latest images
echo "Pulling latest Docker images..."
docker pull $DOCKER_REGISTRY/backend:$VERSION
docker pull $DOCKER_REGISTRY/frontend:$VERSION

# Backup database (production only)
if [[ "$DEPLOY_ENV" == "production" ]]; then
    echo "Creating database backup..."
    docker exec eprice-database pg_dump -U eprice eprice > backup_$(date +%Y%m%d_%H%M%S).sql
fi

# Update docker-compose configuration
if [[ "$DEPLOY_ENV" == "staging" ]]; then
    export COMPOSE_FILE="docker-compose.yml:docker-compose.staging.yml"
else
    export COMPOSE_FILE="docker-compose.yml:docker-compose.production.yml"
fi

# Deploy with zero downtime
echo "Deploying to $DEPLOY_ENV..."

# Start new containers
docker-compose up -d --no-deps --scale backend=2 backend

# Wait for new containers to be healthy
echo "Waiting for new containers to be healthy..."
sleep 30

# Health check
for i in {1..30}; do
    if docker-compose exec backend curl -f http://localhost:8080/health; then
        echo "Health check passed"
        break
    fi
    if [[ $i -eq 30 ]]; then
        echo "Health check failed after 5 minutes"
        exit 1
    fi
    sleep 10
done

# Scale down old containers
docker-compose up -d --no-deps --scale backend=1 backend

# Update frontend
docker-compose up -d frontend

# Run database migrations
echo "Running database migrations..."
docker-compose exec backend ./eprice-server migrate

# Verify deployment
echo "Verifying deployment..."
if ! docker-compose exec frontend wget -q --spider http://localhost/; then
    echo "Frontend health check failed"
    exit 1
fi

if ! docker-compose exec backend curl -f http://localhost:8080/health; then
    echo "Backend health check failed"
    exit 1
fi

# Clean up old images
echo "Cleaning up old Docker images..."
docker image prune -f

echo "Deployment to $DEPLOY_ENV completed successfully!"

# Send notification (if configured)
if [[ -n "$SLACK_WEBHOOK_URL" ]]; then
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"EPrice $DEPLOY_ENV deployment completed successfully! Version: $VERSION\"}" \
        $SLACK_WEBHOOK_URL
fi