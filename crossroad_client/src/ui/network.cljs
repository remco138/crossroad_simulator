(ns ui.network
  (:require
   [cljs.nodejs :as node]))

(def net (node/require "net"))

(enable-console-print!)

(defonce client (atom nil))

(defn clj->json
  [ds]
  (.stringify js/JSON (clj->js ds)))


(defn send! [data]
  (print "data sent: " data)
  (.write @client data))

(defn send-sensor-states! [xs]
  (send! (str (clj->json {:banen (reduce #(conj %1 {:id %2 :bezet true}) [] xs)}) "\n")))

(defn on-receive [data]
  (print data))

(defn on-connect []
  (.write @client "<client>: hello")
  (print "we might have connected to the server and sent our greetings!"))

(defn on-data [data]
  (print data))


(defn connect! [port]
  (print "connecting..")
  (reset! client (.createConnection net port))
  (.on @client "connect" on-connect)
  (.on @client "data" on-data))
