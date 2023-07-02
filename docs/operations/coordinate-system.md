# Coordinate System

When considering the user perspective, we should display the screens in 
physical measurements (centimeter/inches). Because we are allowing users to
draw screens for different devices in our canvas. So the user should get
a clear out-look about the device sizes and how the user's will see those
contents in their devices.

```
+-----------------------------------------------------------------+
|                                                                 |
|                                                                 |
|    +----------------------------------------------------+       |
|    |                     OpenXD                         |       |
|    ------------------------------------------------------       |
|    |         +--------------------------------+         |       |
|    |         |                                |         |       |
|    |         |                                |         |       |
|    |         |           +------------+       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |    O       |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           |            |       |         |       |
|    |         |           +---Screen---+       |         |       |
|    |         |                                |         |       |
|    |         |                                |         |       |
|    |         +--------------Canvas------------+         |       |
|    |                                                    |       |
|    +---------------------Application--------------------+       |
|                                                                 |
+---------------------------Monitor-------------------------------+
```

As in the above visualization, The outside world of the Monitor is using 
physical measurement units such as centimeters. But in the monitor the
measurement unit is pixel. In the canvas we do not have a measurement.
We have a 0-1 coordinate system. But we should emulate a centimeter
coordinate system inside the canvas. In screens, we should emulate
a pixel coordinate system. When we considering multiple screens from
different devices, the pixel sizes should differ.

The most challenging task is we should sync the canvas centimeter coordinate
system with the outside world. As an example, if a user drew a screen with 3
inch height in the canvas, then the user should be able to measure it with
a physical ruler.
