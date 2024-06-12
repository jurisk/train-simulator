variable "gcp_project" {
  description = "The Google Cloud project ID"
  type        = string
}

variable "gcp_zone" {
  description = "The Google Cloud zone"
  type        = string
}

variable "cluster_name" {
  description = "The name of the GKE cluster"
  type        = string
}

variable "static_ip_address" {
  description = "The static IP address for the service"
  type        = string
}

variable "dns_name" {
  description = "The DNS name for the service"
  type        = string
}