# Nibble Mascot Animation Rules

Nibs, Nibble's terminal mascot, is a compact, fixed-width ASCII pet. He should feel
friendly without becoming noisy or physically incoherent.

## Current Shape

Every frame renders as 3 rows by 12 terminal cells:

```txt
      ()_()
     -(o.o)-
#---- (_W_)~
```

The silhouette must stay stable across screens:

- ears stay on the top row
- face stays on the middle row
- body stays on the bottom row
- head, face, and body cores start on the same column
- tail stays after the body on the right side
- `-(...)-` is the mascot's fixed whisker shape, not an arm
- broom/tool movement is detached from the mascot and stays on the left
- idle/search/report screens show the broom lying down as `#----`
- active cleanup uses `/`, `|`, and `\` for detached broom motion
- goodbye uses the same detached broom as a fast celebratory spin
- broom handle characters use the terminal's light-blue ANSI color
- broom bristles `#` use yellow, separate from the handle
- mascot body/outline uses terminal light-blue; eyes and whiskers use white

Do not use `/` or `\` for mascot limbs. Diagonal characters are only allowed for
the detached broom, left of the mascot.

## State Loops

### Happy

Subtle idle loop for home, wizard, empty dashboards, and goodbye:

```txt
      ()_()       ()_()
     -(o.o)-     -(-.-)-
#---- (_W_)~ #---- (_W_)~
```

The blink intentionally lasts longer than a single fast flash so the idle state
reads as a sleepy blink instead of visual noise.

### Search / Scanning / Telemetry

Small dust-dot movement around an otherwise stable mascot:

```txt
      ()_()       ()_()       ()_()
     -(O.o)-     -(o.O)-     -(O.o)-
#---- (_W_)~ #---- (_W_)~ #---- (_W_)~
```

### Sweeping

The broom is separate from the mascot. Product lore: Nibs moves the broom with his mind, so the pet never needs to bend an arm or break silhouette.

```txt
  /   ()_()   |   ()_()   \   ()_()   |   ()_()
 /   -(o.o)-  |  -(o.o)-   \ -(o.o)-  |  -(o.o)-
#     (_W_)~  #   (_W_)~    # (_W_)~  #   (_W_)~
```

### Goodbye / Celebration

Nibs is happy and lets the broom spin separately:

```txt
  /   ()_()   |   ()_()   \   ()_()   |   ()_()
 /   -(^.^)-  |  -(^.^)-   \ -(^.^)-  |  -(^.^)-
#     (_W_)~  #   (_W_)~    # (_W_)~  #   (_W_)~
```

### App Box / Doctor

These are static variants that keep the same silhouette:

```txt
      ()_()       ()_()
     -(o.o)-     -(o.o)-
#---- [_W_]~ #---- (_W_)~
```

## Regression Expectations

The renderer has unit coverage for the core consistency contract:

- every known state renders exactly 3 rows
- every row is padded to 12 cells
- non-broom states do not contain `/` or `\`
- every face row keeps the fixed `-(...)-` whisker silhouette
- head, face, and body cores remain column-aligned
- any visible tail appears after `_W_`
