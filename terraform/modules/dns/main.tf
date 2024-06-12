# Equivalent to `gcloud services enable dns.googleapis.com`
resource "google_project_service" "dns" {
  project = var.gcp_project
  service = "dns.googleapis.com"
  disable_on_destroy = false
}

resource "google_dns_managed_zone" "dns_zone" {
  name        = "ts-krikis-online"
  dns_name    = "ts.krikis.online."
  description = "DNS zone for ts.krikis.online"

  visibility = "public"
  project    = var.gcp_project
}

resource "google_dns_record_set" "a_record" {
  name         = "ts.krikis.online."
  managed_zone = google_dns_managed_zone.dns_zone.name
  type         = "A"
  ttl          = 300
  rrdatas      = [var.ip_address]
  project      = var.gcp_project
}