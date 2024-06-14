# Equivalent to `gcloud services enable compute.googleapis.com`
resource "google_project_service" "compute" {
  project = var.gcp_project
  service = "compute.googleapis.com"
  disable_on_destroy = false
}

resource "google_compute_address" "static_ip" {
  name         = "train-simulator-static-ip"
  project      = var.gcp_project
  region       = var.gcp_region
  address_type = "EXTERNAL"
}

resource "google_compute_network" "default" {
  name = "train-simulator"
}

resource "google_compute_firewall" "default" {
  name    = "train-simulator"
  network = google_compute_network.default.self_link

  allow {
    protocol = "tcp"
    ports    = [var.service_port]
  }

  source_ranges = ["0.0.0.0/0"]
}