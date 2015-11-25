(ns ui.network
  (:require
   [ui.state :as state]
   [cljs.nodejs :as node]))

(defn only-node [f]
  (when-not (undefined? cljs.nodejs) (f)))


(only-node #(def net (node/require "net")))

(enable-console-print!)

(defonce client (atom nil))

(defn clj->json
  [ds]
  (.stringify js/JSON (clj->js ds)))

(defn json->obj [x] (.parse js/JSON x))


(defn send! [data]
  (swap! state/ui-state assoc :last-packet data)
  (print (:last-packet @state/ui-state))
  (.write @client data))


(defn send-sensor-states! [xs]
  (send! (str (clj->json {:banen xs
                          }) "\n")))

(defn on-connect []
  (print "we might have connected to the server and sent our greetings!"))

(defn on-data [data]
  (state/reset-light-states! (.-banen (json->obj data))))


(defn connect!
  ([port]
   (do (print "connecting..")
     (reset! client (.createConnection net port))
     (.on @client "connect" on-connect)
     (.on @client "data" on-data)
     (.setEncoding @client "utf8")
     ))
  ([ip port]
   (do (print "connecting2..")
     (reset! client (.createConnection net port ip))
     (.on @client "connect" on-connect)
     (.on @client "data" on-data))
     (.setEncoding @client "utf8")
   )
  )
