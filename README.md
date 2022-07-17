# Kanidm client profile switcher

This allows you to switch between client config profiles, handy for dev etc.


# Setting your profiles

This all happens in `~/.config/kanidm-profiles.toml`. Here's an example config file:

```t
[[profiles]]
name = "production internal.kanidm.example.com"
uri = "https://internal.kanidm.example.com"
verify_ca = true
verify_hostnames = true

[[profiles]]
name = "test localhost"
uri = "https://localhost:8443"
verify_ca = false
verify_hostnames = false
radius_required_groups = [
        "radius_access_allowed",
]
radius_groups = [
        { name = "radius_access_allowed", vlan = 10 }
]

radius_clients = [
        { name = "test", ipaddr = "127.0.0.1", secret  = "testing123" },
]
```

# TODOs

- add the option to save the current config to a profile
- an "add a new profile" command - ie, add a flag, get prompted for fields
- an edit command - allowing updates to profiles
  