# Armada stick lighting

`armada-stick-lighting` controls stick LEDs exposed through Linux's multicolor
LED interface. Armada supplies the LED names through `device-env`, keeping
device-specific paths out of `armada-control`.

The first version supports a solid color, brightness, persistent off, and
restoring the saved configuration:

```text
armada-stick-lighting get
armada-stick-lighting set --color FF8000 --brightness 25
armada-stick-lighting off
armada-stick-lighting apply
```

Settings are saved to `/etc/armada/stick-lighting.json` after the hardware was
updated successfully. Only LED names declared by the device profile are used.
