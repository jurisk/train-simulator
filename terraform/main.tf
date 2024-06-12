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

module "kubernetes" {
  source            = "./modules/kubernetes"
  gcp_project       = var.gcp_project
  gcp_zone          = var.gcp_zone
  cluster_name      = "ts-gke-cluster"
  static_ip_address = module.network.static_ip_address
  dns_name          = "ts.krikis.online"
}

provider "google" {
  # credentials = file("<path-to-your-service-account-key>.json") # https://developer.hashicorp.com/terraform/tutorials/gcp-get-started/google-cloud-platform-build#authenticate-to-google-cloud
  project = var.gcp_project
  region  = var.gcp_region
}