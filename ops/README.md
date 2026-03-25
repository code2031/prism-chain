# Prism Operations

Production infrastructure, deployment, monitoring, and maintenance tooling for running Prism validators.

## Directory Structure

```
ops/
+-- terraform/          # AWS infrastructure provisioning
|   +-- main.tf         # VPC, EC2 (r6a.8xlarge), EBS (2TB gp3), security groups, CloudWatch alarms
|   +-- variables.tf    # All configurable variables (cluster, instance type, IOPS, CIDRs, etc.)
+-- ansible/            # Server configuration
|   +-- playbook.yml    # Full validator setup: packages, Rust build, kernel tuning, systemd, firewall, monitoring
+-- validator/          # Validator runtime configuration
|   +-- validator-config.sh         # Production launch script with environment-based configuration
|   +-- setup-validator.sh          # Initial validator setup
|   +-- solclone-validator.service  # systemd unit file
+-- monitoring/         # Prometheus + Grafana stack
|   +-- prometheus.yml                  # Scrape config (validator metrics, node exporter)
|   +-- alerts.yml                      # Alerting rules (validator down, delinquent, skip rate, disk, CPU, memory)
|   +-- docker-compose.monitoring.yml   # Prometheus + Grafana + Node Exporter containers
|   +-- grafana/                        # Dashboards and provisioning configs
+-- network/            # Network bootstrap and management
|   +-- bootstrap-network.sh    # Create a new network from genesis
|   +-- add-validator.sh        # Add a validator to an existing network
|   +-- genesis-ceremony.sh     # Multi-party genesis ceremony
+-- backup/             # Snapshot backup
|   +-- snapshot-backup.sh      # S3 snapshot backup with zstd compression and retention pruning
+-- security/           # Security hardening
|   +-- firewall.sh             # UFW firewall configuration
|   +-- hardening.md            # Server hardening guide
+-- runbooks/           # Operational procedures
    +-- incident-response.md    # Incident response playbook
    +-- upgrade.md              # Validator upgrade procedure
```

## Quick Start

### Provision Infrastructure

```bash
cd ops/terraform
terraform init
terraform plan -var="key_pair_name=my-key" -var='ssh_allowed_cidrs=["1.2.3.4/32"]'
terraform apply
```

### Configure Server

```bash
cd ops/ansible
ansible-playbook -i inventory.ini playbook.yml \
  -e cluster=mainnet \
  -e prism_version=1.0.0
```

### Start Monitoring

```bash
cd ops/monitoring
docker compose -f docker-compose.monitoring.yml up -d
# Prometheus: http://localhost:9090
# Grafana:    http://localhost:3000 (admin / prism-monitor)
```

### Bootstrap a Network

```bash
cd ops/network
./bootstrap-network.sh --cluster devnet --identity /path/to/keypair.json
```

## Alert Rules

The monitoring stack includes alerts for: validator down, validator delinquent, high skip rate (>10%), low vote credits, slot lag (>100/>500), disk space (<10%/<20%), high memory (>90%), high CPU (>90%), and RPC latency (p99 >1s).
