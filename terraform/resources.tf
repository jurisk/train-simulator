# Equivalent to `gcloud services enable dns.googleapis.com`
resource "google_project_service" "dns" {
  project            = var.gcp_project
  service            = "dns.googleapis.com"
  disable_on_destroy = false
}

# Equivalent to `gcloud services enable compute.googleapis.com`
resource "google_project_service" "compute" {
  project            = var.gcp_project
  service            = "compute.googleapis.com"
  disable_on_destroy = false
}