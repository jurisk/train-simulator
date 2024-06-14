provider "google" {
  project = "your-gcp-project-id"
  region  = "us-central1"
  zone    = "us-central1-a"
}

# TODO: openssl req -x509 -newkey rsa:2048 -keyout private_key.pem -out certificate.pem -days 365 -nodes -subj "/CN=example.com"
# Import the self-signed certificate and key
resource "tls_private_key" "example" {
  algorithm = "RSA"
  rsa_bits  = 2048
}

resource "tls_self_signed_cert" "example" {
  key_algorithm   = tls_private_key.example.algorithm
  private_key_pem = tls_private_key.example.private_key_pem

  subjects = [
    {
      common_name  = "example.com"
      organization = "Example Inc"
    }
  ]

  validity_period_hours = 8760
  is_ca_certificate     = false
  allowed_uses = [adsf]
}

# Upload certificate to Google Cloud
resource "google_compute_ssl_certificate" "default" {
  name        = "self-signed-cert"
  certificate = tls_self_signed_cert.example.cert_pem
  private_key = tls_private_key.example.private_key_pem
}

# Load balancer resources
resource "google_compute_global_address" "default" {
  name = "lb-ip"
}

resource "google_compute_backend_service" "default" {
  name                  = "backend-service"
  port_name             = "http"
  protocol              = "HTTP"
  timeout_sec           = 10
  load_balancing_scheme = "EXTERNAL"

  backend {
    group = google_compute_instance.vm.network_interface[0].network
  }

  health_checks = [google_compute_health_check.default.self_link]
}

resource "google_compute_health_check" "default" {
  name               = "http-health-check"
  check_interval_sec = 5
  timeout_sec        = 5
  healthy_threshold  = 2
  unhealthy_threshold = 2

  http_health_check {
    port_specification = "USE_SERVING_PORT"
    request_path       = "/"
  }
}

resource "google_compute_url_map" "default" {
  name            = "url-map"
  default_service = google_compute_backend_service.default.self_link
}

resource "google_compute_target_https_proxy" "default" {
  name             = "https-proxy"
  url_map          = google_compute_url_map.default.self_link
  ssl_certificates = [google_compute_ssl_certificate.default.self_link]
}

resource "google_compute_global_forwarding_rule" "default" {
  name       = "https-forwarding-rule"
  target     = google_compute_target_https_proxy.default.self_link
  port_range = "443"
  ip_address = google_compute_global_address.default.address
}
