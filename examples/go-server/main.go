package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"runtime"
	"time"
)

type Response struct {
	Status    string    `json:"status"`
	Timestamp time.Time `json:"timestamp"`
	PID       int       `json:"pid"`
	Uptime    float64   `json:"uptime"`
	Memory    struct {
		Alloc      uint64 `json:"alloc"`
		TotalAlloc uint64 `json:"totalAlloc"`
		Sys        uint64 `json:"sys"`
		NumGC      uint32 `json:"numGC"`
	} `json:"memory"`
	Request struct {
		Method string              `json:"method"`
		URL    string              `json:"url"`
		Header http.Header         `json:"headers"`
	} `json:"request"`
}

var startTime = time.Now()

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "3001"
	}

	host := os.Getenv("HOST")
	if host == "" {
		host = "0.0.0.0"
	}

	addr := fmt.Sprintf("%s:%s", host, port)

	http.HandleFunc("/", handler)

	log.Printf("Server running at http://%s/", addr)
	log.Printf("Process PID: %d", os.Getpid())

	if err := http.ListenAndServe(addr, nil); err != nil {
		log.Fatal(err)
	}
}

func handler(w http.ResponseWriter, r *http.Request) {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)

	response := Response{
		Status:    "ok",
		Timestamp: time.Now(),
		PID:       os.Getpid(),
		Uptime:    time.Since(startTime).Seconds(),
		Request: struct {
			Method string      `json:"method"`
			URL    string      `json:"url"`
			Header http.Header `json:"headers"`
		}{
			Method: r.Method,
			URL:    r.URL.String(),
			Header: r.Header,
		},
	}

	response.Memory.Alloc = m.Alloc
	response.Memory.TotalAlloc = m.TotalAlloc
	response.Memory.Sys = m.Sys
	response.Memory.NumGC = m.NumGC

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)

	log.Printf("[%s] %s %s", time.Now().Format("2006-01-02 15:04:05"), r.Method, r.URL.Path)
}
