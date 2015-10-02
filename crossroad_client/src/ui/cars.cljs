(ns ui.cars
  (:require [cljsjs.paperjs]))


(defn- random-car [roads]
  (str
   (nth (keys roads)
        (-> roads count rand-int)))) ;random number within 0..count(roads)


(def path1 (js/paper.Rectangle. (js-obj
                                 "point" (array 75 75)
                                 "size" (array 75 75)
                                 "strokeColor" "black")))



(def onframe)
