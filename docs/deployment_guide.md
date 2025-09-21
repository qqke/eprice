# EPrice Deployment Guide

## Overview

This guide covers the deployment of the EPrice application to production and staging environments using Docker containers and CI/CD pipelines.

## Architecture

### Production Architecture
```
[Load Balancer] → [Nginx Proxy] → [Frontend Container]
                                → [Backend Container] → [PostgreSQL]
                                                     → [Redis Cache]
```

### Components
- **Frontend**: WASM-based web application served by Nginx
- **Backend**: Rust API server with SQLx database integration
- **Database**: PostgreSQL for persistent data storage
- **Cache**: Redis for session management and caching
- **Proxy**: Nginx for load balancing and SSL termination

## Prerequisites

### System Requirements
- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ RAM
- 20GB+ storage
- SSL certificates (for production)

### Environment Variables
```bash
# Database Configuration
DATABASE_URL=postgresql://eprice:password@localhost:5432/eprice
POSTGRES_PASSWORD=secure_password

# Application Configuration
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Security
JWT_SECRET=your_jwt_secret_here
BCRYPT_ROUNDS=12

# External Services
SLACK_WEBHOOK_URL=https://hooks.slack.com/...
```

## Deployment Methods

### Method 1: Docker Compose (Recommended)

#### Quick Start
```bash
# Clone repository
git clone https://github.com/your-org/eprice.git
cd eprice

# Set environment variables
cp .env.example .env
# Edit .env with your configuration

# Deploy
docker-compose up -d
```

#### Production Deployment
```bash
# Use production compose file
docker-compose -f docker-compose.yml -f docker-compose.production.yml up -d

# Or use deployment script
./scripts/deploy.sh production latest
```

### Method 2: Kubernetes (Enterprise)

#### Prerequisites
- Kubernetes cluster 1.20+
- kubectl configured
- Helm 3.0+

#### Deploy with Helm
```bash
# Add EPrice Helm repository
helm repo add eprice https://charts.eprice.com
helm repo update

# Install
helm install eprice eprice/eprice \
  --namespace eprice \
  --create-namespace \
  --values values.production.yaml
```

## CI/CD Pipeline

### GitHub Actions Workflow

The CI/CD pipeline includes:
1. **Testing**: Multi-platform test execution
2. **Security**: Vulnerability scanning with cargo-audit
3. **Build**: Docker image creation
4. **Deploy**: Automated deployment to staging/production
5. **Monitoring**: Health checks and notifications

### Pipeline Stages

#### 1. Test Stage
- Runs on Ubuntu, Windows, and macOS
- Executes unit and integration tests
- Validates code formatting and linting
- Generates coverage reports

#### 2. Build Stage
- Creates optimized Docker images
- Pushes to container registry
- Tags with commit SHA and version

#### 3. Deploy Stage
- Staging deployment on develop branch
- Production deployment on main branch
- Zero-downtime deployment strategy
- Automatic rollback on failure

### Triggering Deployments

#### Automatic Deployment
- **Staging**: Triggered by pushes to `develop` branch
- **Production**: Triggered by pushes to `main` branch

#### Manual Deployment
```bash
# Deploy specific version to staging
./scripts/deploy.sh staging v1.2.3

# Deploy to production
./scripts/deploy.sh production v1.2.3
```

## Configuration Management

### Environment-Specific Configurations

#### Staging
```yaml
# docker-compose.staging.yml
version: '3.8'
services:
  backend:
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=postgresql://eprice:staging_pass@staging-db:5432/eprice_staging
```

#### Production
```yaml
# docker-compose.production.yml
version: '3.8'
services:
  backend:
    environment:
      - RUST_LOG=warn
      - DATABASE_URL=postgresql://eprice:prod_pass@prod-db:5432/eprice_prod
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
```

### Secrets Management

#### Using Docker Secrets
```bash
# Create secrets
echo "database_password" | docker secret create db_password -
echo "jwt_secret_key" | docker secret create jwt_secret -

# Reference in compose file
services:
  backend:
    secrets:
      - db_password
      - jwt_secret
```

#### Using External Secret Management
- **AWS**: Secrets Manager or Parameter Store
- **Azure**: Key Vault
- **GCP**: Secret Manager
- **Kubernetes**: Secrets and ConfigMaps

## Monitoring and Logging

### Health Checks

#### Application Health Endpoints
- **Backend**: `GET /health`
- **Frontend**: `GET /`
- **Database**: Connection validation

#### Docker Health Checks
```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
```

### Logging Configuration

#### Centralized Logging
```yaml
# docker-compose.yml
services:
  backend:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

#### Log Aggregation
- **ELK Stack**: Elasticsearch, Logstash, Kibana
- **Grafana**: Grafana, Loki, Promtail
- **Cloud Solutions**: CloudWatch, Azure Monitor, Stackdriver

### Metrics Collection

#### Prometheus Integration
```rust
// Add to backend
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref HTTP_REQUESTS: Counter = register_counter!(
        "http_requests_total", "Total HTTP requests"
    ).unwrap();
}
```

## Backup and Recovery

### Database Backups

#### Automated Backups
```bash
# Daily backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
docker exec eprice-database pg_dump -U eprice eprice > backup_$DATE.sql
aws s3 cp backup_$DATE.sql s3://eprice-backups/
```

#### Point-in-Time Recovery
```bash
# Restore from backup
docker exec -i eprice-database psql -U eprice eprice < backup_20231201_120000.sql
```

### Application Data Backup
```bash
# Backup uploaded files and data
docker run --rm -v eprice_backend_data:/data \
  -v $(pwd):/backup alpine tar czf /backup/data_backup.tar.gz /data
```

## Security Considerations

### SSL/TLS Configuration

#### Certificate Management
```nginx
# nginx-proxy.conf
server {
    listen 443 ssl http2;
    ssl_certificate /etc/ssl/certs/eprice.crt;
    ssl_certificate_key /etc/ssl/private/eprice.key;
    
    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
}
```

#### Let's Encrypt Integration
```bash
# Certbot setup
docker run -it --rm --name certbot \
  -v "/etc/letsencrypt:/etc/letsencrypt" \
  -v "/var/lib/letsencrypt:/var/lib/letsencrypt" \
  certbot/certbot certonly --webroot \
  -w /var/lib/letsencrypt/ \
  -d eprice.com
```

### Network Security

#### Firewall Configuration
```bash
# UFW rules
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw allow 22/tcp    # SSH (admin only)
ufw deny 5432/tcp   # PostgreSQL (internal only)
```

#### Docker Network Isolation
```yaml
# docker-compose.yml
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true
```

## Scaling Strategies

### Horizontal Scaling

#### Load Balancing
```yaml
# docker-compose.yml
services:
  backend:
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
```

#### Database Scaling
- **Read Replicas**: For read-heavy workloads
- **Connection Pooling**: PgBouncer for connection management
- **Sharding**: For very large datasets

### Vertical Scaling

#### Resource Limits
```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
        reservations:
          cpus: '1.0'
          memory: 1G
```

## Troubleshooting

### Common Issues

#### 1. Container Won't Start
```bash
# Check logs
docker logs eprice-backend

# Check resource usage
docker stats

# Verify configuration
docker-compose config
```

#### 2. Database Connection Issues
```bash
# Test database connectivity
docker exec eprice-backend nc -zv eprice-database 5432

# Check database logs
docker logs eprice-database
```

#### 3. High Memory Usage
```bash
# Monitor memory usage
docker stats --no-stream

# Analyze memory leaks
docker exec eprice-backend ps aux --sort=-%mem
```

### Performance Optimization

#### 1. Docker Image Optimization
- Use multi-stage builds
- Minimize image layers
- Remove unnecessary dependencies

#### 2. Database Optimization
- Add proper indexes
- Optimize query performance
- Configure connection pooling

#### 3. Caching Strategy
- Implement Redis caching
- Use CDN for static assets
- Enable browser caching

## Rollback Procedures

### Automatic Rollback
```bash
# Health check failure triggers automatic rollback
if ! curl -f http://localhost:8080/health; then
  ./scripts/rollback.sh production $PREVIOUS_VERSION
fi
```

### Manual Rollback
```bash
# Rollback to previous version
./scripts/rollback.sh production v1.2.2

# Verify rollback
curl -f http://localhost:8080/health
```

## Maintenance

### Regular Maintenance Tasks

#### Weekly
- Review logs for errors
- Check disk space usage
- Verify backup integrity
- Update security patches

#### Monthly
- Update dependencies
- Review performance metrics
- Clean up old Docker images
- Rotate log files

#### Quarterly
- Security audit
- Disaster recovery testing
- Performance optimization review
- Documentation updates

### Upgrade Procedures

#### Minor Version Upgrades
```bash
# Pull new images
docker-compose pull

# Rolling update
docker-compose up -d --no-deps backend
```

#### Major Version Upgrades
1. Schedule maintenance window
2. Create full backup
3. Test upgrade in staging
4. Execute upgrade with rollback plan
5. Verify all functionality
6. Monitor for issues

## Contact and Support

### Emergency Contacts
- **Operations Team**: ops@eprice.com
- **Development Team**: dev@eprice.com
- **On-call Engineer**: +1-555-EPRICE

### Documentation
- **API Documentation**: https://docs.eprice.com/api
- **User Guide**: https://docs.eprice.com/user
- **Technical Documentation**: https://docs.eprice.com/tech

### Support Channels
- **Slack**: #eprice-ops
- **Tickets**: https://support.eprice.com
- **Wiki**: https://wiki.eprice.com