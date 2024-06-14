module "dns" {
  source      = "./modules/dns"
  gcp_project = var.gcp_project
  ip_address  = module.network.static_ip_address
}

module "network" {
  source       = "./modules/network"
  gcp_project  = var.gcp_project
  gcp_region   = var.gcp_region
  service_port = var.service_port
}

module "compute" {
  source            = "./modules/compute"
  gcp_project       = var.gcp_project
  gcp_zone          = var.gcp_zone
  static_ip_address = module.network.static_ip_address
  service_port      = var.service_port
  network_name      = module.network.network_name
}

provider "google" {
  # credentials = file("<path-to-your-service-account-key>.json") # https://developer.hashicorp.com/terraform/tutorials/gcp-get-started/google-cloud-platform-build#authenticate-to-google-cloud
  project = var.gcp_project
  region  = var.gcp_region
}