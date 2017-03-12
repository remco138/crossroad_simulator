(ns ui.core
  (:require
   [figwheel.client :as fw :include-macros true]
   [reagent.core :as reagent :refer [atom]]
   [clojure.string :as string :refer [split-lines]]
   [cljsjs.paperjs]

   [ui.network :as network]
   [ui.state :as state]
   [ui.drawing :as drawing]
   [ui.cars :as cars]
   [ui.sensors :as sensors]
   ))

(enable-console-print!)



(defn parse-svg [url]
  (let [item js/paper.project]
    (print (.importSVG item url))))

(defn spawn-car! []
  (let [car (state/add-car-channel! (cars/random-car　(:roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))
(defn spawn-walker! []
  (let [car (state/add-car-channel! (cars/random-walker　(:pedestrian-roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))
(defn spawn-bus! []
  (let [car (state/add-car-channel! (cars/random-bus　(:bus-roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))
(defn spawn-cyclist! []
  (let [car (state/add-car-channel! (cars/random-cyclist　(:cycling-roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))

(defn root-component []
  [:div
   [:canvas {:id "mycanvas" :width "967" :height "459"}]
   [:textarea {:id "logger":readOnly true :rows 5 :cols 30 :placeholder  "logger" :value (:last-packet @state/ui-state)}]
   [:br]
   [:button  {:on-click spawn-car!} "spawn car"]
   [:button  {:on-click spawn-walker!} "spawn pedestrian"]
   [:button  {:on-click spawn-bus!} "spawn bus"]
   [:button  {:on-click spawn-cyclist!} "spawn cyclist"]
   [:br]
   [:span "spawn every: "]
   [:input {:type "range"
            :value (:spawn-rate @state/ui-state)
            :min 0.1
            :max 5
            :step 0.1
            :name "rate-slider"
            :on-change (fn [x]
                         (swap! state/ui-state assoc :spawn-rate (-> x .-target .-value js/parseFloat)))}]
   [:br]
   [:button  {:on-click #(network/connect! (:connect-port @state/ui-state))} "(re)connect"]
   [:button  {:on-click #(network/send! "ayyyyy\r\n")} "send data"]
   [:br]

   [:input {:type "text"
            :value (:connect-ip @state/ui-state)
            :name "ip"
            :on-change (fn [x]
                         (swap! state/ui-state assoc :connect-ip (-> x .-target .-value)))}]
   [:input {:type "text"
            :value (:connect-port @state/ui-state)
            :name "port"
            :on-change (fn [x]
                         (swap! state/ui-state assoc :connect-port (-> x .-target .-value)))}]
   [:button {:on-click #(network/connect! (@state/ui-state :connect-ip) (@state/ui-state :connect-port)  )}
    "connect!"]

   [:span "show sensors"]
   [:input {:type "checkbox"
            :checked (:display-sensors @state/ui-state)
            :name "displaySensors"
            :on-change (fn [x] (swap! state/ui-state assoc :display-sensors (-> x .-target .-checked)))}]
   [:span "show paths"]
   [:input {:type "checkbox"
            :checked (:display-paths @state/ui-state)
            :name "displaySensors"
            :on-change (fn [x] (swap! state/ui-state assoc :display-paths (-> x .-target .-checked)))}]

   [:br]
   [:span "car spee multiplier "]
   [:input {:type "range"
            :value (:speed @state/ui-state)
            :min 0.1
            :max 10
            :step 0.1
            :name "speed-slider"
            :on-change (fn [x]
                         (swap! state/ui-state assoc :speed (-> x .-target .-value js/parseFloat)))}]
   [:span (:speed @state/ui-state)]
      [:br]
   [:span "sensor refresh rate "]
      [:input {:type "range"
            :value (:sensor-refresh @state/ui-state)
            :min 100
            :max 2000
            :step 1
            :name "sensor-refresh-rate"
            :on-change (fn [x]
                         (swap! state/ui-state assoc :sensor-refresh (-> x .-target .-value js/parseInt)))}]
   [:span (:sensor-refresh @state/ui-state)]

  ])

(reagent/render
  [root-component]
  (.-body js/document))


(defn init! []
  (js/paper.setup(js/document.getElementById "mycanvas"))
  (set! js/paper.view.onFrame (var drawing/on-frame))
  ;(network/connect!  "127.0.0.1" 9990)
  
  (swap! state/ui-state assoc :connect-port "9080")
  (state/init!)
  (drawing/init!)
  (sensors/track-sensors!))

(set! (.-onload js/window) (fn [] (init!)))


(fw/watch-and-reload
  :websocket-url   "ws://localhost:3449/figwheel-ws"
  :jsload-callback (fn []
                     (init!)
                     (print "reloaded")))

