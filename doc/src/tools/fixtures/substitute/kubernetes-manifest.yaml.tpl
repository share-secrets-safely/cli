apiVersion: v1
data:
  game.properties: |
    enemies=aliens
    lives=3
    enemies.cheat=true
  ui.properties: |
    color.good=purple
    color.bad=yellow
kind: ConfigMap
metadata:
  name: game-config
  namespace: default
  labels:
    team: {{team.name}}
    department: {{team.department}}
    project: {{project.name}}
    kind: {{project.kind}}
