# Equivalent to `gcloud services enable compute.googleapis.com`
resource "google_project_service" "compute" {
  project = var.gcp_project
  service = "compute.googleapis.com"
}

resource "google_compute_address" "static_ip" {
  name         = "ts-static-ip"
  project      = var.gcp_project
  region       = var.gcp_region
  address_type = "EXTERNAL"
}