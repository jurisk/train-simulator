resource "google_compute_address" "static_ip" {
  name         = "train-simulator-static-ip"
  project      = var.gcp_project
  region       = var.gcp_region
  address_type = "EXTERNAL"
}

resource "google_compute_network" "default" {
  name = "train-simulator"
}

resource "google_compute_firewall" "allow-http" {
  name    = "train-simulator-firewall-http"
  network = google_compute_network.default.self_link

  allow {
    protocol = "tcp"
    ports    = [var.service_port]
  }

  source_ranges = ["0.0.0.0/0"]
}

resource "google_compute_firewall" "allow-ssh" {
  name    = "train-simulator-firewall-ssh"
  network = google_compute_network.default.self_link

  allow {
    protocol = "tcp"
    ports    = [22]
  }

  source_ranges = ["0.0.0.0/0"]
}