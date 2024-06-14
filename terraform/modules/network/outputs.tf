output "static_ip_address" {
  value = google_compute_address.static_ip.address
}

output "network_name" {
  value = google_compute_network.default.name
}