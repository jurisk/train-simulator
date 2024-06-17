resource "google_dns_managed_zone" "dns_zone" {
  name        = "ts-krikis-online"
  dns_name    = "ts.krikis.online."
  description = "DNS zone for ts.krikis.online"

  visibility = "public"
  project    = var.gcp_project

  dnssec_config {
    kind  = "dns#managedZoneDnsSecConfig"
    state = "on"
  }

  # We pointed the NS records to the right Google Cloud DNS servers, and if we destroy, these links will be broken
  lifecycle {
    prevent_destroy = true
  }
}

resource "google_dns_record_set" "proxy_server_a_record" {
  name         = "ts.krikis.online."
  managed_zone = google_dns_managed_zone.dns_zone.name
  type         = "A"
  ttl          = 60 # Later: Increase
  rrdatas      = [var.proxy_server_ip_address]
  project      = var.gcp_project
}

resource "google_dns_record_set" "game_server_a_record" {
  name         = "game-server.ts.krikis.online."
  managed_zone = google_dns_managed_zone.dns_zone.name
  type         = "A"
  ttl          = 60 # Later: Increase
  rrdatas      = [var.game_server_ip_address]
  project      = var.gcp_project
}

resource "google_dns_record_set" "caa_letsencrypt" {
  name         = "ts.krikis.online."
  managed_zone = google_dns_managed_zone.dns_zone.name
  type         = "CAA"
  ttl          = 3600
  rrdatas      = ["0 issue \"letsencrypt.org\""]
  project      = var.gcp_project
}