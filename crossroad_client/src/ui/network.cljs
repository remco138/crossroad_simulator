(ns ui.network
  (:require
   [ui.state :as state]
   [cljs.nodejs :as node]))

(defn only-node [f]
  (when (exists? js/window.process) (f)))

(defn nodejs? [] (exists? js/window.process))

(only-node #(def net (node/require "net")))

(enable-console-print!)

(defonce client (atom nil))

(defn clj->json
  [ds]
  (.stringify js/JSON (clj->js ds)))

(defn json->obj [x]
  (.parse js/JSON x))


(defn send! [data]
  (swap! state/ui-state assoc :last-packet data)
  ;(print (:last-packet @state/ui-state))
  (when-not (nil? @client)
    (if (nodejs?)
      (.write @client data)
      (.send @client data))))


(defn send-sensor-states! [xs]
  (send! (str (clj->json {:banen xs})
              "\n")))

(defn on-connect []
  (print "we might have connected to the server and sent our greetings!"))

(defn on-data [data]
(print "onmessage")
(.log js/console data)
  (when (< 5 (.-length data))
    (let [until (.search data "\r\n")
          splitted (.substring data 0 until)]
      (print "splitted: " splitted "\n")
      (state/reset-light-states! (.-stoplichten (json->obj splitted)))
      (print "SLICED"(.slice data until))
      (on-data (.slice data (inc until))))
    ))


(defn connect-websocket!
  ([port]
   (do (print "connecting websocket..")
     (reset! client (js/WebSocket. (str "ws://127.0.0.1:" port)))
     (set! (.-binaryType @client) "arraybuffer")
     ;(.on @client "connect" on-connect)
     (set! (.-onmessage @client) #(on-data (.decode (js/TextDecoder.) (.-data %))))
      ;encoding is utf8 by default for websockets
     ))
  ([ip port]
   (do (print "connecting2 websocket..")
     (reset! client (js/WebSocket. (str "ws://" ip ":" port)))
     (set! (.-binaryType @client) "arraybuffer")
     ;(.on @client "connect" on-connect)
     (set! (.-onmessage @client) #(on-data (.decode (js/TextDecoder.) (.-data %))))
      ;encoding is utf8 by default for websockets
   )
  )
)


(defn connect-socket!
  ([port]
   (do (print "connecting socket..")
     (reset! client (.createConnection net port))
     (.on @client "connect" on-connect)
     (.on @client "data" on-data)
     (.setEncoding @client "utf8")
     ))
  ([ip port]
   (do (print "connecting2 socket..")
     (reset! client (.createConnection net port ip))
     (.on @client "connect" on-connect)
     (.on @client "data" on-data)
     (.setEncoding @client "utf8")
   )
  )
)

(defn connect!
  ([port]
   (if (nodejs?)
     (connect-socket! port)
     (connect-websocket! port)))
  ([ip port]
   (if (nodejs?)
     (connect-socket! port)
     (connect-websocket! ip port))))
