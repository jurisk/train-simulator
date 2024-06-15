module "dns" {
  source      = "./modules/dns"
  gcp_project = var.gcp_project
  # Later: For not using a static IP address, but updating DNS A records to the instance IP address
  #   proxy_server_ip_address  = module.network.static_ip_address
  game_server_ip_address  = module.game-server.public_ip_address
  proxy_server_ip_address = module.proxy.public_ip_address
}

module "network" {
  source       = "./modules/network"
  gcp_project  = var.gcp_project
  gcp_region   = var.gcp_region
  service_port = var.service_port
}

module "game-server" {
  container_image = "gcr.io/train-simulator-gcp/train-simulator:latest"
  source          = "./modules/compute"
  project_id      = var.gcp_project
  network_name    = module.network.network_name
  zone            = var.gcp_zone
  instance_name   = "train-simulator-game-server-vm"
  env_vars = {
    "RUST_BACKTRACE" = "full",
    "RUST_LOG"       = "info",
  }
}

module "proxy" {
  container_image = "gcr.io/train-simulator-gcp/train-simulator-proxy:latest"
  source          = "./modules/compute"
  project_id      = var.gcp_project
  network_name    = module.network.network_name
  zone            = var.gcp_zone
  instance_name   = "train-simulator-proxy-vm"
}

provider "google" {
  # credentials = file("<path-to-your-service-account-key>.json") # https://developer.hashicorp.com/terraform/tutorials/gcp-get-started/google-cloud-platform-build#authenticate-to-google-cloud
  project = var.gcp_project
  region  = var.gcp_region
}