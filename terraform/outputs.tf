output "static_ip_address" {
  value = module.network.static_ip_address
}

output "cluster_endpoint" {
  value = module.kubernetes.cluster_endpoint
}

output "cluster_ca_certificate" {
  value = module.kubernetes.cluster_ca_certificate
}