Overview
========
This example Linux application uses RPMSG (via shared memory) to provide a way to create a console log file of the output from code running on the Cortex-M4. The code connects to the code 
running on the Cortex-M4 via a named channel and address that needs to match that specified in the Cortex-M4 firmware (see https://git.research.arm.com/attk/hybrid/smart-camera-hybrid-application/-/tree/main/rpmsg_cortexm_console_rtos) 


Building
========

This code can be compiled natively on the EVK board:

 gcc -o cortexm_console cortexm_console.c


or the Dockerfile can be used to cross-compile

 docker buildx build --platform linux/arm64 -o output -f Dockerfile .


To use this application alongside the hybrid runtime the output binary 'cortexm_console' should be put in the /usr/local/bin directory on the EVK board


Execution
=========

The program takes a single (optional) argument setting the pathname of the logfile to be written to. If no argument is provided then the log file is written to ```/tmp/m_console.log```

The firmware must be started on the Cortex-M4 first using the remoteproc functionality.

The firmware will wait for the cortexm_console Linux application to start which will complete the handshake and then execution of the firmware will continue.
Data written by the firmware using the rprintf macro will be sent to the Linux application and then written into the logfile.

The Linux application can be run as a background process.

If the firmware is stopped then the Linux application will also stop cleanly.

If the Linux application is stopped while the firmware is still executing then any data from the Cortex-M4 will result in a kernel log message 

  virtio_rpmsg_bus virtio0: msg received with no recipient

The firmware should continue to run.  In this situation it is possible to rerun the Linux application and re-connect (specify a different logfile if you want to retain the information sent from the first invocation)





