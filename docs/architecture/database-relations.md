# Database Relations

     +------------------+                                                                                                          
     |    Session       |      +----------------+         +----------------+                                                       
     --------------------      |     Tab        |         |   Tab Action   |                                                       
     |User              |      ------------------         ------------------                                                       
     |Created At        |      |Session         |         |Tab             |                                                       
     |Exited At         |      |Created At      |         |Action          |                                                       
     +------------------+      |Exited At       |         |Created Time    |                                                       
                               |Head            |         +----------------+                                                       
                               |Snapshot        |                                                                                  
                               |Branch          |                                                                                  
                               +----------------+                                                                                  
    +------------------+                                                                                                           
    |      Ticket      |       +-----------------+        +-----------------+                                                      
    --------------------       |   Stash         |        |  Stash Action   |                                                      
    | User             |       -------------------        -------------------                                                      
    | Last Session     |       |Tab              |        |Stash            |                                                      
    | Created At       |       |Head             |        |Action           |                                                      
    |                  |       |Created At       |        |Created Time     |                                                      
    |                  |       |Snapshot         |        |                 |                                                      
    +------------------+       |Branch           |        |                 |                                                      
                               +-----------------+        +-----------------+                                                      
-----------------------------------------------------------------------------------------------------------------------------------
                                                                                                                                   
                                                                                                                                   
            +----------------+                                                                                                     
            |      User      |             +-----------------+    +----------------+     +------------------+                      
            ------------------             |     Project     |    |   Branch       |     |    Commit        |                      
            | Name           |             -------------------    ------------------     --------------------                      
            | Created At     |             | Name            |    |Name            |     |Message           |                      
            |                |             | Slug            |    |Created At      |     |Created At        |                      
            |                |             | Created At      |    |Init Snapshot   |     |Head              |                      
            |                |             | User            |    |Head            |     |Branch            |                      
            |                |             +-----------------+    |Current Snapshot|     |User              |                      
            +----------------+                                    +----------------+     |                  |                      
                                                                                         +------------------+                      
                                                                                                                                   
                                               +-------------+                          +----------------+                         
                                               |    Tag      |                          | Commit Action  |                         
                                               ---------------                          ------------------                         
                                               |Name         |                          |Commit          |                         
                                               |Created At   |                          |Action          |                         
                                               |Head         |                          |Created Time    |                         
                                               |Snapshot     |                          |                |                         
                                               +-------------+                          +----------------+                         
