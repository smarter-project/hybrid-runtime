FROM gcc:latest as builder

COPY cortexm_console.c cortexm_console.c

RUN gcc -o cortexm_console cortexm_console.c

FROM scratch
COPY --from=builder cortexm_console /


