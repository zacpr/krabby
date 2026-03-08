# The Plan: #
Codenamed 'rusty_containers' , the goal here is to create an utility in Rust that can be installed on a modern Linux system (inital focus on Fedora and KDE).
The utility will provide a system tray menu and and GUI application to allow for the management of containers (docker initially, but with a view to take on podman later, and perhaps others as apporpriate.
I envision the system tray menu being the central element, when clicked the user  should be presented with a sortable list of the containers resident, wihc shows their status, stats and other primary details in a configurable tabular format and allows for each to be stopped, started, restarted etc.
In additional functionality; 
 - the container informaiton table should be exportable as csv or json
 - the user can select one or more containers and then have a docker compose (or podman compose) file generated
 - the user should be able to opt to have ech container whose image is available in a repository regualrly checked to see if updates are available.
 - any other features that could be helpful in these kinds of scenarios.
The appliction should focus on presenting this informtion and the asociated actions in a clear, concise manner, however it should alsobe noted that it is the year 2026, so t is also important that the interface is attractive, engaging and customizable. theuser shoudl be able to adjsut tabular readouts in terms of columns and the appliction of filters and the inteface itself should be able to ahve its color theme modified and where hardware aceleration exists, some shadowing, animation and particle efects etc will help to engage. 
If time permits, incuding a facility to re-create docker run containers, or alter docker compose files through an elegnt GUI would be highly regarded. 
The utility needs a snazzy name, and a corresponding cool logo, but I;ll leve tht i your capable hands.
The utlity should be packaged for proper appication install in Linux
As son as aaviable mnimum roduct is acheived it should be added to my giuthub acocunt as a private project, and github actions should be established to automate the packaging and testing of new versions.
