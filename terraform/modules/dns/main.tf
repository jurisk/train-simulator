# Equivalent to `gcloud services enable dns.googleapis.com`
resource "google_project_service" "dns" {
  project = var.gcp_project
  service = "dns.googleapis.com"
}

resource "google_dns_managed_zone" "dns_zone" {
  name        = "ts-krikis-online"
  dns_name    = "ts.krikis.online."
  description = "DNS zone for ts.krikis.online"

  visibility = "public"
  project    = var.gcp_project
}
