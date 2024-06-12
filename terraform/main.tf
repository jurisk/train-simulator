module "dns" {
  source      = "./modules/dns"
  gcp_project = var.gcp_project
  ip_address  = module.network.static_ip_address
}

module "network" {
  source      = "./modules/network"
  gcp_project = var.gcp_project
  gcp_region  = var.gcp_region
}

provider "google" {
  # credentials = file("<path-to-your-service-account-key>.json") # https://developer.hashicorp.com/terraform/tutorials/gcp-get-started/google-cloud-platform-build#authenticate-to-google-cloud
  project = var.gcp_project
  region  = var.gcp_region
}