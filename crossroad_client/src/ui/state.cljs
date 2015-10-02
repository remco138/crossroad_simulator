(ns ui.state
  (:require [clojure.string :as string :refer [split-lines]]))

;example state
;(def state (atom {:roads {:road1 {:traffic-light "light1", :intersections [[1,1] [666,666] [999,999]]}
 ;                         :road2 {:traffic-light "light2", :intersections [[444,444]]}}}))


(defn generate! []
  (println "implement me :("))
