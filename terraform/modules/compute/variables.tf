variable "gcp_project" {
  description = "The Google Cloud project ID"
  type        = string
}

variable "gcp_zone" {
  description = "The Google Cloud zone"
  type        = string
}

variable "static_ip_address" {
  description = "The static IP address for the service"
  type        = string
}

variable "service_port" {
  description = "The port the service listens on"
  type        = number
}

variable "network_name" {
  description = "The name of the network"
  type        = string
}